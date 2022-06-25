mod elements;
mod field;
mod ubresenham;

use std::error::Error;
use std::sync::{Arc, Mutex, mpsc};
use std::{thread, cmp};
use std::time::{Instant, Duration};

use elements::movable_solids::MovableSolid;
use elements::{Element};
use field::Field;
use field::chunk::CHUNK_SIZE;
use field::rect::Rect;
use ubresenham::Ubresenham;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent, VirtualKeyCode, MouseScrollDelta};
use winit::window::WindowBuilder;
use winit::event_loop::{EventLoop, ControlFlow};
use pixels::{SurfaceTexture, Pixels};
use winit_input_helper::WinitInputHelper;

enum InputMessage{
    Click(f32, f32),
    RClick(f32, f32),
    Scroll(isize),
    Number(usize),
    MousePosition(f32, f32),
}
const CHUNK_NUMBER: (usize, usize) = (8, 8);

const SCALE_FACTOR: u32 = 6;

const THREAD_NUMBER: usize = 16;

const DRAW_BOXES: bool = true;

const FPS: f32 = 120.;

fn main() -> Result<(), Box<dyn Error>> {
    
    let elements = [||Element::wet_sand(), ||Element::sand(), ||Element::water(), ||Element::oil(), ||Element::block()];
    
    let mut element_index:usize = 1;

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let pix_number = ((CHUNK_NUMBER.0 * CHUNK_SIZE.0) as u32, (CHUNK_NUMBER.1 * CHUNK_SIZE.1) as u32);
    let max_inner_size: PhysicalSize<u32> = pix_number.into();
    let window_size: PhysicalSize<u32> = (pix_number.0 * SCALE_FACTOR, pix_number.1 * SCALE_FACTOR).into();

    let window = WindowBuilder::new()
        .with_fullscreen(None)
        .with_inner_size(window_size)
        .with_max_inner_size(max_inner_size)
        .with_title("wgpu first steps")
        .build(&event_loop)
        .unwrap();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let pixels = Pixels::new(pix_number.0, pix_number.1, surface_texture)?;
    let pixels_access = Arc::new(Mutex::new(pixels));
    let pix_for_draw = pixels_access.clone();
    let pix_for_change = pixels_access.clone();



    let (sender, receiv) = mpsc::channel();

    thread::spawn(move || { 
        let mut prev_spawn_cord: Option<(usize, usize)> = None;
        let mut field = Field::new(CHUNK_NUMBER, THREAD_NUMBER);
        let mut previus_chunks = Vec::new();
        let mut previus_rects = Vec::new();
        let mut brush_size: (usize, usize) = (3,3);
        
        let mut mouse_prev: Option<(usize, usize)> = None;
        let mut brush_size_prev: (usize, usize) = brush_size;
        loop{
            
            let loop_start = Instant::now();
            window.request_redraw();
            field.update();
            let mut inputs = Vec::new();
            while let Ok(input) = receiv.try_recv(){
                inputs.push(input);
            }
            {
                let mut pix = pix_for_change.lock().unwrap();

                let mut spawn = false;

                let mut spawn_element = None;

                let mut spawn_cord = (0., 0.);
                
                let mut mouse_position = None;
                for input in inputs.into_iter(){
                    match input{
                        InputMessage::Click(x, y) => {
                            spawn = true;
                            spawn_cord = (x, y);
                            spawn_element = Some(elements[element_index]());
                        },
                        InputMessage::Number(index) => {
                            element_index = index % elements.len();
                        },
                        InputMessage::RClick(x, y) => {
                            spawn = true;
                            spawn_cord = (x, y);
                            spawn_element = None;
                        },
                        InputMessage::Scroll(v) => {
                            brush_size = (cmp::min(10, cmp::max(1, brush_size.0 as isize + v)) as usize, 
                            cmp::min(10, cmp::max(1, brush_size.1 as isize + v)) as usize)
                        },
                        InputMessage::MousePosition(x, y) => {
                            if let Ok(pos) = pix.window_pos_to_pixel((x, y)){
                                mouse_position = Some(pos);
                            }
                        },
                    }
                }

                if spawn {
                    let mut spawn_func = |cord: (usize, usize)| 
                    {field.set_in_area((cord.0 as isize, cord.1 as isize), brush_size, spawn_element)};
                    if let Ok(cord) = pix.window_pos_to_pixel(spawn_cord){
                        if let Some(prev_cord) = prev_spawn_cord{

                            if cord == prev_cord{
                                spawn_func(cord);
                            }
                            else{
                                let line = Ubresenham::new(prev_cord, cord);
                                for cord in line{
                                    spawn_func(cord);
                                }
                            }
                        }
                        else{
                            spawn_func(cord);
                        }
                        prev_spawn_cord = Some(cord);
                    }
                }
                else{
                    prev_spawn_cord = None;
                }

                let frame = pix.get_frame();

                let color_func = |p| { match field.get(p){
                    Some(e) => e.get_color(),
                    None => [0x00,0x00,0x00,0xff],
                }};

                if let Some(mouse_prev) = mouse_prev{
                    draw_adaptive_color_rect(frame, 
                        Rect::from_center((mouse_prev.0 as isize, mouse_prev.1 as isize), brush_size_prev), 
                        color_func);
                }

                if DRAW_BOXES{
                    let color_func = |p| { match field.get(p){
                        Some(e) => e.get_color(),
                        None => [0x00,0x00,0x00,0xff],
                    }};
                    for b in previus_chunks.iter(){
                        draw_adaptive_color_rect(frame, *b, color_func);
                    }
                    for b in previus_rects.iter(){
                        draw_adaptive_color_rect(frame, *b, color_func);
                    }
                }
                
                update_frame(frame, &mut field);

                if DRAW_BOXES{
                    previus_chunks.clear();
                    previus_rects.clear();
                    for b in field.get_chunks(){
                        draw_rect(frame, b, &[0xff, 0x00, 0x00, 0xff]);
                        previus_chunks.push(b);
                    }
                    for b in field.get_chunks_update_rects(){
                        draw_rect(frame, b, &[0x00, 0xff, 0x00, 0xff]);
                        previus_rects.push(b);
                    }
                }

                if let Some(mouse_pos) = mouse_position{
                    
                    draw_rect(frame, Rect::from_center((mouse_pos.0 as isize, mouse_pos.1 as isize), brush_size), &[0xff, 0xff, 0xff, 0xff]);
                }

                mouse_prev = mouse_position;
                brush_size_prev = brush_size;

            }
            let duration_sec = (Instant::now() - loop_start).as_secs_f32();
            let wait_time = (1. / FPS) - duration_sec;
            if wait_time > 0. {
                thread::sleep(Duration::from_secs_f32(wait_time));
            }
        }
    });
    let mut can_send = Box::new(false);
    let mut can_send_mouse = false;
    event_loop.run(move |event, _, control_flow| {

        match event {
            Event::RedrawRequested(_) => {
                {
                    *can_send = true;
                    can_send_mouse = true;
                    let pixels = pix_for_draw.lock().unwrap();
                    pixels.render().ok();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { 
                event: WindowEvent::MouseWheel { 
                     delta: MouseScrollDelta::LineDelta(_, delta) ,
                     ..
                },
                ..
            } => {
                let val = delta.signum() as isize;
                if *can_send && val != 0 {
                    *can_send = false;
                    sender.send(InputMessage::Scroll(delta.signum() as isize)).ok();
                }
            }
            
            _ => {}
        }

        if input.update(&event) {
            if can_send_mouse {
                if let Some(p) = input.mouse(){
                    can_send_mouse = false;
                    sender.send(InputMessage::MousePosition(p.0, p.1)).ok();
                }
            }
            if *can_send {
                if let Some(p) = input.mouse(){
                    if (input.mouse_held(0)){
                        *can_send = false;
                        sender.send(InputMessage::Click(p.0, p.1)).ok();
                    }
                    else if (input.mouse_held(1)){
                        *can_send = false;
                        sender.send(InputMessage::RClick(p.0, p.1)).ok();
                    }
                }
                
                for (key_index, code) in [
                    VirtualKeyCode::Key0,
                    VirtualKeyCode::Key1,
                    VirtualKeyCode::Key2,
                    VirtualKeyCode::Key3,
                    VirtualKeyCode::Key4,
                    VirtualKeyCode::Key5,
                    VirtualKeyCode::Key6,
                    VirtualKeyCode::Key7,
                    VirtualKeyCode::Key8,
                    VirtualKeyCode::Key9,
                    ].into_iter().enumerate(){
                    if input.key_pressed(code){
                        *can_send = false;
                        sender.send(InputMessage::Number(key_index)).ok();
                    }
                }
            }
        }
    });
    Ok(())
}

fn convert_cords(cord: (isize, isize)) -> Option<usize>{
    if cord.0 < 0 || cord.1 < 0 || cord.0 >= (CHUNK_NUMBER.0 * CHUNK_SIZE.0) as isize || 
    cord.1 >= (CHUNK_NUMBER.1 * CHUNK_SIZE.1) as isize{
        return None;
    }
    let cord = (cord.0 as usize, cord.1 as usize);
    Some(cord.1 * CHUNK_NUMBER.0 * CHUNK_SIZE.0 + cord.0)
}

fn update_frame(frame: &mut [u8], field: &mut Field){
    for (pix_cord, color) in field.load_pixels(){
        if let Some(index) = convert_cords(pix_cord){
            let pixel = &mut frame[index*4..(index+1)*4];
            pixel[0] = color[0];
            pixel[1] = color[1];
            pixel[2] = color[2];
            pixel[3] = color[3];
        }
    }
}

fn set_pix(frame: &mut [u8], pix: (isize, isize), color: &[u8; 4]){
    let index = convert_cords(pix);
    if index.is_none(){
        return;
    }
    let index = index.unwrap();
    let pixel_buf = &mut frame[index*4..(index+1)*4];

    pixel_buf[0] = color[0];
    pixel_buf[1] = color[1];
    pixel_buf[2] = color[2];
    pixel_buf[3] = color[3];
}

fn draw_adaptive_color_rect(frame: &mut [u8], rect: Rect<isize>, color_func: impl Fn((isize, isize)) -> [u8; 4]){
    for x in rect.left()..rect.right(){
        set_pix(frame, (x, rect.top()), &color_func((x, rect.top())));
        set_pix(frame, (x, rect.bottom()-1), &color_func((x, rect.bottom()-1)));
    }
    for y in rect.top()..rect.bottom(){
        set_pix(frame, (rect.left(), y), &color_func((rect.left(), y)));
        set_pix(frame, (rect.right() - 1, y), &color_func((rect.right() - 1, y)));
    }
}

fn draw_rect(frame: &mut [u8], rect: Rect<isize>, color: &[u8; 4]){
    for x in rect.left()..rect.right(){
        set_pix(frame, (x, rect.top()), color);
        set_pix(frame, (x, rect.bottom()-1), color);
    }
    for y in rect.top()..rect.bottom(){
        set_pix(frame, (rect.left(), y), color);
        set_pix(frame, (rect.right() - 1, y), color);
    }
}
