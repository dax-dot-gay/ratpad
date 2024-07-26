import usb_cdc
import supervisor

# supervisor.runtime.autoreload = False

usb_cdc.enable(console=True, data=True)
usb_cdc.data.timeout = 0
