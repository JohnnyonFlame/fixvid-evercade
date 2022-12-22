extern crate drm;

use std::num::NonZeroU32;
use drm::Device;
use drm::control::{Device as ControlDevice, encoder, framebuffer};
use drm::control::{connector, crtc};
use std::os::unix::io::{AsRawFd, RawFd};
use std::fs::{OpenOptions, File};
use std::process::ExitCode;

struct Card(File);
impl Device for Card {}
impl ControlDevice for Card {}

impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

fn main() -> ExitCode {
    // Basic IO setup
    let mut options = OpenOptions::new();
    options.read(true);
    options.write(true);

    let card = Card(options.open("/dev/dri/card0").unwrap());

    // Query necessary DRM information
    let res = card
        .resource_handles()
        .expect("fixvid: Could not load normal resource ids.");

    let coninfo: Vec<connector::Info> = res.connectors()
        .iter()
        .flat_map(|con| card.get_connector(*con, true))
        .collect();

    let crtcinfo: Vec<crtc::Info> = res.crtcs()
        .iter()
        .flat_map(|crtc| card.get_crtc(*crtc))
        .collect();

    let encinfo: Vec<encoder::Info> = res.encoders()
        .iter()
        .flat_map(|enc| card.get_encoder(*enc))
        .collect();
    
    // Check if we have any connector that is current connected to something
    let con = coninfo
        .iter()
        .find(|&i| i.state() == connector::State::Connected)
        .expect("fixvid: No connected connectors");    

    // Check if this connector has a valid pipe
    let found_pipe = crtcinfo.iter().any(|crtc| {
        let _done = (|| -> Option<bool> {
            let enc = encinfo.iter().find(|i| i.crtc().is_some() && i.crtc().unwrap() == crtc.handle())?;
            let _con = coninfo.iter().find(|i| i.current_encoder().is_some() && i.current_encoder().unwrap() == enc.handle())?;

            Some(true)
        })();

        _done.is_some()
    });

    // We have nothing to fix, bail
    if found_pipe {
        println!("fixvid: Nothing to do.");
        return ExitCode::from(0)
    }

    // Haven't bailed...
    let rebuilt_pipe = (|| -> Option<bool> {
        // Find new pipe (crtc -> encoder -> connector)
        // connector -> encoder
        let new_enc = card.get_encoder(*con.encoders().first()?).unwrap();
        // encoder -> crtc
        let new_crtc = card.get_crtc(*res.filter_crtcs(new_enc.possible_crtcs()).first()?).unwrap();

        // Search for the missing framebuffer
        for fb in 1 .. 255 {
            // Try all modes, first modes wins...
            for mode in con.modes() {
                let done = card.set_crtc(new_crtc.handle(), Some(framebuffer::Handle::from(NonZeroU32::new(fb).unwrap())), (0, 0), &[con.handle()], Some(*mode));
                if done.is_ok() {
                    println!("fixvid: Rewired lost framebuffer {} with mode {:?}", fb, mode.name());
                    return Some(true)
                }
            }
        }

        None
    })();

    if rebuilt_pipe.is_none() {
        println!("fixvid: Unable to rebuild pipe...");
        return ExitCode::from(255);
    }

    ExitCode::from(0)
}
