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

  //Getting the current time is actually an expensive operation in web browsers, so I only do it in the refresh cycle.
  let mut previous_duration = instant::Duration::new(0, 11_000);  //11 microseconds
  
  loop {
    let start_time = instant::Instant::now();
    sleep(&window, 16).await;  //The printer flips every 28 ms, however we need to signal the processor that it was switched, on / off, so we run at 16 ms, so switch on and off.
    let cycles = previous_duration.as_nanos() / 10_800; //cpu runs at 10.3 microseconds per opcode. Each cycle is 8 clock ticks at 740 khz.
    for _ in 0..cycles {
      board.run_cycle();
      keyboard.run_cycle(&mut board);
      printer.run_cycle(&mut board);
    }
    
    keyboard.run_sleep_cycle(&mut board);
    printer.run_sleep_cycle(&mut board);

    side_panel::print_memory(&board);
    side_panel::print_ports(&board);
    side_panel::print_shifts(&board);
    
    //Getting the current time is actually an expensive operation in web browsers, so I only do it in the refresh cycle.
    previous_duration = instant::Instant::now() - start_time;
  }
}

async fn sleep(window: &web_sys::Window, ms: i32) {
  let promise = js_sys::Promise::new(&mut |resolve, _| {
    window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms).unwrap();
  });
  let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}
