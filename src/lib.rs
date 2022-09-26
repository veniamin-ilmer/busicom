//! Testing using the official MCS4 Evaluation Kit with 4001-0009
//! Useful resources: http://8bitforce.com/projects/4004/

#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;

use boards::busicom141pf::Board;

use instant;

mod side_panel;
mod keyboard;
mod printer;

/// ```
/// Loop {
///   Sleep till the next screen refresh
///   Calculate how much instructions we should have run.
///   Run those instructions.
///   Run instructions until the calculated cycles matches the actual cycles.
///   Run IO instructions which should only work on each refresh
///   Draw everything.
/// }
/// ```
#[wasm_bindgen]
pub async fn run() {
  std::panic::set_hook(Box::new(console_error_panic_hook::hook)); //Panics appear more descriptive in the browser console.

  let mut board = Board::new();
  
  let mut keyboard = keyboard::Keyboard::new();
  let mut printer = printer::Printer::new();

  let window = web_sys::window().unwrap();
  let mobile = window.match_media("(hover: none)").unwrap().unwrap().matches();
  let sleep_time = if mobile {
    16 * 16 //4 hz refresh
  } else {
    16  //60 hz refresh
  };


  loop {
    let start_time = instant::Instant::now();
    sleep(&window, sleep_time).await;  //The printer flips every 28 ms, however we need to signal the processor that it was switched, on / off, so we run at 16 ms, so switch on and off.
    let mut actual_cycles = 0;
    let duration = instant::Instant::now() - start_time;
    let calculated_cycles = duration.as_nanos() / 10_800; //cpu runs at 10.3 microseconds per opcode. Each cycle is 8 clock ticks at 740 khz.
    loop {
//      let calculated_cycles = duration.as_millis() / 16; //Slowest speed.. ~60Hz
//      let calculated_cycles = 1;
      if actual_cycles >= calculated_cycles {
        break;
      }
      board.run_cycle();
      keyboard.run_cycle(&mut board);
      printer.run_cycle(&mut board);
      actual_cycles += 1;
    }
    
    keyboard.run_sleep_cycle(&mut board);
    printer.run_sleep_cycle(&mut board);

    //Run a few more times, since we are sleeping longer
    if mobile {
      for _ in 0..15 {
        keyboard.run_sleep_cycle(&mut board);
        printer.run_sleep_cycle(&mut board);
      }
    }
    
    if !mobile {
      side_panel::print_memory(&board);
      side_panel::print_ports(&board);
      side_panel::print_shifts(&board);
    }
  }
}

async fn sleep(window: &web_sys::Window, ms: i32) {
  let promise = js_sys::Promise::new(&mut |resolve, _| {
    window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms).unwrap();
  });
  let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}
