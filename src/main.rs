#![windows_subsystem = "windows"]
extern crate conrod;
extern crate rosc;
extern crate serde_json;

use std::env;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::net::{ SocketAddrV4, UdpSocket };
use std::str::FromStr;
use std::fs::File;
use std::io::BufReader;

use serde::{ Deserialize, Serialize };

use conrod::color;
use conrod::backend::glium::glium::{ self, Surface };
use glium::glutin as gg;

use rosc::{ encoder, OscMessage, OscPacket, OscType };
use ntwk_tester::ui::{ Slider, Text, Line };

const FONT_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/Helvetica400.ttf");

#[derive(Serialize, Deserialize, Debug)]
struct SliderVal {
    default: f64,
    min: f64,
    max: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct OscProfile {
    rcv_addr: String,
    snd_addr_fr: String,
    snd_addr_to: String,
    snd_path: String,
    snd_args: Vec<SliderVal>
}

#[derive(Serialize, Deserialize, Debug)]
struct Profile {
    width: u32,
    height: u32,
    osc_profile: OscProfile,
}

fn main() {
    let reader = BufReader::new( File::open("profile.json").unwrap());
    let profile: Profile = serde_json::from_reader(reader).unwrap();
    let w = profile.width as f64;
    let h = profile.height as f64;
    
    let mut event_loop = gg::EventsLoop::new();
    let mut ui = conrod::UiBuilder::new([w, h]).build();
    ui.fonts.insert_from_file(FONT_PATH).unwrap();
    awake_osc(profile.osc_profile, &mut ui, &mut event_loop, w, h)
}

fn awake_osc(profile: OscProfile, ui: &mut conrod::Ui, event_loop: &mut gg::EventsLoop, w: f64, h: f64){
    let rcv_addr = match SocketAddrV4::from_str(&profile.rcv_addr)    { Ok(addr) => addr, Err(_) => panic!() };
    let snd_addr = match SocketAddrV4::from_str(&profile.snd_addr_fr) { Ok(addr) => addr, Err(_) => panic!() };
    let rcv_sock = UdpSocket::bind(rcv_addr).unwrap();
    let snd_sock = UdpSocket::bind(snd_addr).unwrap();
    let ids = &mut ntwk_tester::Ids::new(ui.widget_id_generator());
    
    let window = gg::WindowBuilder::new().with_title("osc-network-tester").with_dimensions((w, h).into());
    let context = gg::ContextBuilder::new().with_vsync(true).with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
    let img_map = conrod::image::Map::<glium::texture::Texture2d>::new();
    
    ids.sliders.resize(profile.snd_args.len(), &mut ui.widget_id_generator());
    let mut output = Text::new(ids.output, "", 50.0, h * 0.5 + 30.0, w - 100.0, 100.0);
    let mut sender = Text::new(ids.sender, "", 50.0, 30.0,           w - 100.0, 100.0);
    let mut line_h = Line::new(ids.line_h, color::DARK_GRAY);
    
    let mut sliders = Vec::<Slider>::new();
    for (i, s) in ids.sliders.iter().enumerate() {
        let x = 50.0 + i as f64 * 60.0;
        let a = &profile.snd_args[i];
        sliders.push(Slider::new(*s, a.default, a.min, a.max, x, 90.0, 20.0, 120.0));
    };

    let received = Arc::new(Mutex::new(Vec::<rosc::OscMessage>::new()));
    {
        let received = received.clone();
        let mut buf = [0u8; rosc::decoder::MTU];
        let _ = thread::spawn( move || { 
            loop {
                match rcv_sock.recv_from(&mut buf) {
                    Ok((size, ..)) => {
                        let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                        let mut r = received.lock().unwrap();
                        if let Some(v) = handle_packet(packet){
                            while r.len() > 6 { r.remove(0); }
                            r.push(v);
                        }
                    }
                    Err(e) => { println!("{}", e); break; }
                }
            }
        });
    }
    
    event_loop.run_forever(|event : gg::Event| {
            
        // sender
        let values: Vec<f64> = sliders.iter().map(|s| s.value).collect();
        let sender_info = format!("Sending from {}\nMessage: {} {} [{:.3?}]",
            profile.snd_addr_fr,
            profile.snd_addr_to,
            profile.snd_path,
            values 
        );
        
        let buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: profile.snd_path.clone(),
            args: values.iter().map(|v| OscType::Float(*v as f32)).collect()
        })).unwrap();
        
        snd_sock.send_to(&buf, &profile.snd_addr_to).unwrap();
        
        // reciever
        let received = received.lock().unwrap();
        let mut debug = format!("Listening to {}\nReceived packets:\n", rcv_addr);
        for v in received.iter().rev() {
            debug.push_str(&format!("{} : {:.3?}\n", v.addr, v.args));
        }
        
        match event.clone() {
            gg::Event::WindowEvent { event, .. } => match event {
                gg::WindowEvent::CloseRequested => return gg::ControlFlow::Break,
                _ => (),
            },
            _ => ()
        }

       // input handler
       if let Some(input) = conrod::backend::winit::convert_event(event, &display) {
           ui.handle_event(input);
        };
        
        let mut ui = ui.set_widgets();
        sender.update(&mut ui, &sender_info);
        output.update(&mut ui, &debug);
        for s in sliders.iter_mut(){ s.update(&mut ui); }
        line_h.update(&mut ui, w * -0.5, 0.0, w * 0.5, 0.0);
        
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &img_map);
            let mut target = display.draw();
            target.clear_color(0.3, 0.3, 0.3, 1.0);
            renderer.draw(&display, &mut target, &img_map).unwrap();
            target.finish().unwrap();
        }
        
        gg::ControlFlow::Continue
    });
}

fn handle_packet(packet: OscPacket) -> Option<rosc::OscMessage> {
    match packet {
        OscPacket::Message(msg) => Some(msg),
        OscPacket::Bundle(..) => None // undefined
     }
}