use crate::*;

use crate::drivers::virtio::*;

use super::structs::*;
use super::consts::*;

/// VirtIO GPU Driver
pub struct GPUDriver
{
    pub device: VirtIODeviceDriver,
    frame_buffer: Framebuffer
}

impl GPUDriver
{
    /// Create a new gpu driver from a device driver
    pub fn new(device: VirtIODeviceDriver) -> Self
    {
        if device.get_device_type() != VirtIODeviceType::GPUDevice
        {
            panic!("Cannot create GPU device from {:?}", device.get_device_type());
        }

        Self
        {
            device,
            frame_buffer: Framebuffer::new(640, 480)
        }
    }

    /// Perform the device specific initialization
    pub fn device_specific(&mut self, _features: u32) -> Result<(), String>
    {
        self.device.verify_queue_size()?;

        self.device.init_queues(2)?;

        self.device.driver_ok();

        Ok(())
    }

    /// Send a request to the driver
    fn send_request<RqT, RpT: Default>(&mut self, rq: *mut Request<RqT, RpT>)
    {
        let desc0 = VirtIODescriptor
        {
            addr: unsafe { &(*rq).request as *const RqT as u64 },
            len: core::mem::size_of::<RqT>() as u32,
            flags: VIRTIO_DESC_F_NEXT,
            next: 0,
        };

        let desc1 = VirtIODescriptor
        {
            addr: unsafe { &(*rq).response as *const RpT as u64 },
            len: core::mem::size_of::<RpT>() as u32,
            flags: VIRTIO_DESC_F_WRITE,
            next: 0,
        };

        let head = self.device.add_descriptor_to_queue(0, desc0);
        self.device.add_descriptor_to_queue(0, desc1);

        self.device.send_on_queue(0, head);
    }

    /// Send a request to the driver
    fn send_request3<RqT, RmT, RpT: Default>(&mut self, rq: *mut Request3<RqT, RmT, RpT>)
    {
        let desc0 = VirtIODescriptor
        {
            addr: unsafe { &(*rq).request as *const RqT as u64 },
            len: core::mem::size_of::<RqT>() as u32,
            flags: VIRTIO_DESC_F_NEXT,
            next: 0,
         };

         let desc1 = VirtIODescriptor
         {
            addr: unsafe { &(*rq).mementries as *const RmT as u64 },
            len: core::mem::size_of::<RmT>() as u32,
            flags: VIRTIO_DESC_F_NEXT,
            next: 0,
         };

         let desc2 = VirtIODescriptor
         {
            addr: unsafe { &(*rq).response as *const RpT as u64 },
            len: core::mem::size_of::<RpT>() as u32,
            flags: VIRTIO_DESC_F_WRITE,
            next: 0,
         };

        let head = self.device.add_descriptor_to_queue(0, desc0);
        self.device.add_descriptor_to_queue(0, desc1);
        self.device.add_descriptor_to_queue(0, desc2);

        self.device.send_on_queue(0, head);
    }

    /// Initialize the driver (clear the frame buffer)
    pub fn init(&mut self)
    {
        let (width, height) = self.frame_buffer.get_size();

        self.send_request(Request::<ResourceCreate2d, CtrlHeader>::new(ResourceCreate2d
            {
            hdr: CtrlHeader
            {
               ctrl_type: CtrlType::CmdResourceCreate2d,
               flags: 0,
               fence_id: 0,
               ctx_id: 0,
               padding: 0,
            },
            resource_id: 1,
            format: Formats::R8G8B8A8Unorm,
            width: width as u32,
            height: height as u32,
        }));

        self.send_request3(Request3::<AttachBacking, MemEntry, CtrlHeader >::new(AttachBacking {
            hdr: CtrlHeader
            {
               ctrl_type: CtrlType::CmdResourceAttachBacking,
               flags: 0,
               fence_id: 0,
               ctx_id: 0,
               padding: 0,
            },
            resource_id: 1,
            nr_entries: 1,
            },
            MemEntry
            {
                addr: self.frame_buffer.data as u64,
                length: (width * height * core::mem::size_of::<Pixel>()) as u32,
                padding: 0, 
            }
        ));

        self.send_request(Request::<SetScanout, CtrlHeader>::new(SetScanout {
            hdr: CtrlHeader {
               ctrl_type: CtrlType::CmdSetScanout,
               flags: 0,
               fence_id: 0,
               ctx_id: 0,
               padding: 0,
            },
            r: Rect::new(0, 0, width as u32, height as u32),
            resource_id: 1,
            scanout_id: 0,
        }));
    }

    /// Invalidate and transfer part of the frame buffer
    pub fn invalidate(&mut self, x: usize, y: usize, width: usize, height: usize)
    {
        self.send_request(Request::<TransferToHost2d, CtrlHeader>::new(TransferToHost2d
            {
            hdr: CtrlHeader
            {
                ctrl_type: CtrlType::CmdTransferToHost2d,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            r: Rect::new(x as u32, y as u32, width as u32, height as u32),
            offset: 0,
            resource_id: 1,
            padding: 0,
            }));

        self.send_request( Request::<ResourceFlush, CtrlHeader>::new(ResourceFlush
            {
            hdr: CtrlHeader
            {
               ctrl_type: CtrlType::CmdResourceFlush,
               flags: 0,
               fence_id: 0,
               ctx_id: 0,
               padding: 0,
            },
            r: Rect::new(x as u32, y as u32, width as u32, height as u32),
            resource_id: 1,
            padding: 0,
         }));
    }

    
}