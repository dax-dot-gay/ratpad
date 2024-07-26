import usb_cdc

usb_cdc.enable(console=True, data=True)
usb_cdc.data.timeout = 0
