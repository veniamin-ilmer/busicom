use arbitrary_int::u4;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use boards::busicom141pf::Board;

pub(super) struct Keyboard {
  current_scan_code: u8,
  pending_click_var: wasm_bindgen::JsValue,
  timer: u8,
  html_document: web_sys::Document,
  digit_precision: u8,
  rounding: u8,
}

impl Keyboard {
  pub(super) fn new() -> Self {
    let pending_click_var = js_sys::Reflect::get(
        &wasm_bindgen::JsValue::from(web_sys::window().unwrap()),
        &wasm_bindgen::JsValue::from("getPendingClick"),
    ).unwrap();
    
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    
    Self {
      current_scan_code: 0,
      pending_click_var,
      timer: 0,
      html_document: document,
      digit_precision: 8,
      rounding: 0,  //"Floating" mode.
    }
  }
  
  /// Set ROM bit depending on current shifter bit
  pub(super) fn run_cycle(&mut self, board: &mut Board) {
    let shifter = board.i4003s[0].read_parallel();
    let key_out = u4::new(match (shifter, self.current_scan_code) {
      (0b0000000001, 129) => 0b0001, //CM
      (0b0000000001, 130) => 0b0010, //RM
      (0b0000000001, 131) => 0b0100, //M-
      (0b0000000001, 132) => 0b1000, //M+
      (0b0000000010, 133) => 0b0001, //SQRT
      (0b0000000010, 134) => 0b0010, //%
      (0b0000000010, 135) => 0b0100, //M=-
      (0b0000000010, 136) => 0b1000, //M=+
      (0b0000000100, 137) => 0b0001, //diamond
      (0b0000000100, 138) => 0b0010, //divide
      (0b0000000100, 139) => 0b0100, //times
      (0b0000000100, 140) => 0b1000, //equals
      (0b0000001000, 141) => 0b0001, //minus
      (0b0000001000, 142) => 0b0010, //plus
      (0b0000001000, 143) => 0b0100, //diamond2
      (0b0000001000, 144) => 0b1000, //000
      (0b0000010000, 145) => 0b0001, //9
      (0b0000010000, 146) => 0b0010, //6
      (0b0000010000, 147) => 0b0100, //3
      (0b0000010000, 148) => 0b1000, //.
      (0b0000100000, 149) => 0b0001, //8
      (0b0000100000, 150) => 0b0010, //5
      (0b0000100000, 151) => 0b0100, //2
      (0b0000100000, 152) => 0b1000, //00
      (0b0001000000, 153) => 0b0001, //7
      (0b0001000000, 154) => 0b0010, //4
      (0b0001000000, 155) => 0b0100, //1
      (0b0001000000, 156) => 0b1000, //0
      (0b0010000000, 157) => 0b0001, //Sign
      (0b0010000000, 158) => 0b0010, //EX
      (0b0010000000, 159) => 0b0100, //CE
      (0b0010000000, 160) => 0b1000, //C
      (0b0100000000, _) => self.digit_precision & 0xF,
      (0b1000000000, _) => self.rounding & 0xF,
      _ => 0,
    });
    board.i4001s[1].write_ports(key_out);
  }
  
  pub(super) fn run_sleep_cycle(&mut self, board: &mut Board) {
    self.handle_keypress(board);
    self.handle_switches();
    self.handle_leds(board);
  }
  
  fn handle_keypress(&mut self, board: &mut Board) {
    if self.timer > 0 {
      self.timer -= 1;
      return;
    }
    if self.current_scan_code == 0 {
      if let Some(scan_code) = self.get_keypress() {
        self.current_scan_code = scan_code;
        match self.current_scan_code {
          0 => (),
          1 => {  //Advance paper
            let mut rom_ports = board.i4001s[2].read_ports();
            rom_ports |= u4::new(0b1000);  //Advance paper signal
            board.i4001s[2].write_ports(rom_ports);
            self.timer = 1;
          },
          _ => {  //actual scan code will be handled by fast run cycle due to it being dependent on shift register.
            self.timer = 1;
          },
        }
      }
    } else {
      //Undo button press
      match self.current_scan_code {
        1 => {  //Advance paper
          let mut rom_ports = board.i4001s[2].read_ports();
          rom_ports &= u4::new(0b0111);  //Advance paper signal
          board.i4001s[2].write_ports(rom_ports);
          self.timer = 7;
        },
        _ => {
          self.timer = 1;
        },
      }
      self.current_scan_code = 0;
    }
  }
  
  fn get_keypress(&self) -> Option<u8> {
    let pending_click_func: &js_sys::Function = self.pending_click_var.dyn_ref().unwrap();
    let click_var = pending_click_func.apply(&JsValue::null(), &js_sys::Array::new()).unwrap();
    if let Some(click_float) = click_var.as_f64() {
      Some(click_float.round() as u8) //Wish there were a way to get an integer directly without needing to go through a float...
    } else {
      None
    }
  }
  
  fn handle_switches(&mut self) {
    let digits_element = self.html_document.get_element_by_id("digits").unwrap();
    let digits: &web_sys::HtmlInputElement = digits_element.dyn_ref().unwrap();
    self.digit_precision = digits.value().parse().unwrap();
    
    let float_element = self.html_document.get_element_by_id("float").unwrap();
    let float: &web_sys::HtmlInputElement = float_element.dyn_ref().unwrap();
    if float.checked() {
      self.rounding = 0;
      return;
    }
    let round_element = self.html_document.get_element_by_id("round").unwrap();
    let round: &web_sys::HtmlInputElement = round_element.dyn_ref().unwrap();
    if round.checked() {
      self.rounding = 1;
      return;
    }
    let truncate_element = self.html_document.get_element_by_id("truncate").unwrap();
    let truncate: &web_sys::HtmlInputElement = truncate_element.dyn_ref().unwrap();
    if truncate.checked() {
      self.rounding = 8;
      return;
    }
  }
  
  fn handle_leds(&mut self, board: &mut Board) {
    let ram_ports = board.i4002s[1].read_ports().value();

    self.update_led("memory_led", if ram_ports & 0b1 == 0b1 { "lightblue" } else { "none" });
    self.update_led("overflow_led", if ram_ports & 0b10 == 0b10 { "#ffcccb" } else { "none" });
    self.update_led("negative_led", if ram_ports & 0b100 == 0b100 { "lightgreen" } else { "none" });
  }

  fn update_led(&mut self, led_id: &str, color: &str) {
    let led = self.html_document.get_element_by_id(led_id).unwrap();
    let _ = led.set_attribute("style", &format!("background-color: {}", color));
  }
}
