import os
import re
import shutil
import unicodedata
import rich.progress
from watchdog.observers import Observer
from watchdog.events import FileSystemEvent, FileSystemEventHandler
import click
from circup.command_utils import find_device
import tomllib


class WatchHandler(FileSystemEventHandler):
    def __init__(self, board_path: str, project_path: str) -> None:
        super().__init__()
        self.board_path = board_path
        self.project_path = project_path
        self.progress = None
        self.progress_count = 0
        self.task = None

    def start_progress(self):
        self.progress_count = 0
        total = sum([len(i[2]) for i in os.walk(self.project_path)])
        print(total)
        self.progress = rich.progress.Progress()
        self.task = self.progress.add_task("Copying files...", total=total)
        self.progress.start()

    def stop_progress(self):
        self.progress_count = 0
        self.progress.stop()
        self.progress = None
        self.task = None

    def copy_hook(self, *args, **kwargs):
        if self.progress:
            self.progress_count += 1
            self.progress.console.print(f"[b]Copying: [/b]{args[0]}")
            self.progress.update(self.task, completed=self.progress_count)
        return shutil.copy2(*args, **kwargs)

    def deploy(self):
        self.start_progress()
        shutil.copytree(
            self.project_path,
            self.board_path,
            copy_function=self.copy_hook,
            dirs_exist_ok=True,
        )
        self.stop_progress()

    def on_any_event(self, event: FileSystemEvent) -> None:
        if event.event_type in ["opened", "closed"]:
            return
        self.deploy()


def slugify(value, allow_unicode=False):
    value = str(value)
    if allow_unicode:
        value = unicodedata.normalize("NFKC", value)
    else:
        value = (
            unicodedata.normalize("NFKD", value)
            .encode("ascii", "ignore")
            .decode("ascii")
        )
    value = re.sub(r"[^\w\s-]", "", value.lower())
    return re.sub(r"[-\s]+", "-", value).strip("-_").replace("-", "_")


@click.command
@click.option(
    "--target",
    "-t",
    "target",
    type=click.Path(file_okay=False, dir_okay=True, exists=True),
    help="Path to CIRCUITPY drive",
    default=None,
)
@click.option(
    "--path",
    "-p",
    "path",
    type=click.Path(file_okay=False, dir_okay=True, exists=True),
    help="Path to code directory",
    default=None,
)
@click.option(
    "--deploy", "-d", "deploy", is_flag=True, help="Whether to just write once"
)
def main(path: str | None, target: str | None, deploy: bool):
    resolved_target = target if target else find_device()
    if not resolved_target:
        raise RuntimeError(
            "No path was specified & circup was unable to automatically locate a board to write to."
        )

    if path:
        resolved_path = path
    else:
        if not os.path.exists("pyproject.toml"):
            raise RuntimeError(
                "No pyproject.toml was found in the root project directory"
            )

        with open("pyproject.toml", "rb") as f:
            data = tomllib.load(f)

        project_name = data.get("project", {}).get("name", None)
        if not project_name:
            raise RuntimeError("Unknown project name")

        resolved_path = os.path.join("src", slugify(project_name))
        if not os.path.exists(resolved_path):
            raise RuntimeError("Cannot locate project path")

    if deploy:
        WatchHandler(resolved_target, resolved_path).deploy()
    else:
        watcher = Observer()
        watcher.schedule(
            WatchHandler(resolved_target, resolved_path), resolved_path, recursive=True
        )
        watcher.start()
        try:
            while watcher.is_alive():
                watcher.join(1)
        finally:
            watcher.stop()
            watcher.join()


if __name__ == "__main__":
    main()
