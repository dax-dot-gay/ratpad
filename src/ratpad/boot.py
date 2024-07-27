import usb_cdc
import supervisor
import storage

supervisor.runtime.autoreload = False
storage.remount("/", readonly=False, disable_concurrent_write_protection=False)

usb_cdc.enable(console=True, data=True)
usb_cdc.data.timeout = 0
