use arbitrary_int::u4;
use chips::mcs4;
use chips::mcs4::shifter4003;

pub(super) struct Printer {
  spin_count: usize,  //There are a total of 13 rotations. However, we need to signal the processor that it was switched, on / off, so we have 26 rotations, switching on and off.
  red_color: bool,
  printer_rows: web_sys::HtmlCollection,
  paper_rows: web_sys::HtmlCollection,
  advance_paper: bool,
  hammering: bool,
}

impl Printer {
  pub(super) fn new() -> Self {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    
    Self {
      spin_count: 0,
      red_color: false,
      printer_rows: {
        let table = document.get_element_by_id("printer").unwrap();
        let tbody = table.children().item(0).expect("can't get tbody");
        tbody.children()
      },
      paper_rows: {
        let table = document.get_element_by_id("paper").unwrap();
        let tbody = table.children().item(0).expect("can't get tbody");
        tbody.children()
      },
      advance_paper: false,
      hammering: false,
    }
  }
  
  pub(super) fn run_sleep_cycle(&mut self, board: &mut mcs4::Board) {
    if self.spin_count % 2 == 1 {
      board.cpu.set_test_flag(false);
    } else {
      board.cpu.set_test_flag(true);
      self.spin_printer();
      
      let mut rom_ports = board.roms[2].ports;
      if self.spin_count == 26 {
        rom_ports |= u4::new(0b1);  //Every full rotation of the printer, we set this to 1.
        self.spin_count = 0;
      } else {
        rom_ports &= u4::new(0b1110);
      }
      board.roms[2].ports = rom_ports;
    }
    self.spin_count += 1;
  }
  
  pub(super) fn run_cycle(&mut self, board: &mcs4::Board, shifters: &[mcs4::shifter4003::Shifter]) {
    let ram0_data = board.rams[0].ports;
    if self.new_advance_paper_signal(ram0_data) {
      self.move_up_paper();
    }
    if self.new_hammer_signal(ram0_data) {
      self.hit_hammer(printer_shift_bits(shifters));
    }

    //Red color change is per page, not per character. Once set, it can't be changed until you go to the next line.
    if ram0_data.value() & 0b1 == 0b1 {
      self.red_color = true;
    }
  }

  fn spin_printer(&self) {
    let cells = self.printer_rows.item(0).expect("can't get tr").children();
    self.write_digits(cells, 0b11111111111111111111);
  }

  fn write_digits(&self, td_list: web_sys::HtmlCollection, bits: u32) {
    let mut html_index = 0;
    for index in 3..=17 {
      if (bits >> index) & 1 == 1 {
        if let Some(td) = td_list.item(html_index) {
          td.set_text_content(Some(match self.spin_count / 2 - 1 {
            0 => "0", 1 => "1", 2 => "2", 3 => "3", 4 => "4", 5 => "5", 6 => "6",
            7 => "7", 8 => "8", 9 => "9", 10 => ".", 11 => ".", _ => "-",
          }));
          if self.red_color {
            td.set_class_name("red");
          } else {
            td.set_class_name("");
          }
        }
      }
      html_index += 1;
    }
    if bits & 1 == 1 {
      if let Some(td) = td_list.item(16) {
        td.set_text_content(Some(match self.spin_count / 2 - 1 {
          0 => "◇", 1 => "+", 2 => "-", 3 => "×", 4 => "÷", 5 => "M+", 6 => "M-",
          7 => "^", 8 => "=", 9 => "√", 10 => "%", 11 => "C", _ => "R",
        }));
        if self.red_color {
          td.set_class_name("red");
        } else {
          td.set_class_name("");
        }
      }
    }
    if (bits >> 1) & 1 == 1 {
      if let Some(td) = td_list.item(17) {
        td.set_text_content(Some(match self.spin_count / 2 - 1 {
          0 => "#", 1 => "*", 2 => "\u{2160}", 3 => "\u{2161}", 4 => "\u{2162}", 5 => "M+", 6 => "M-",
          7 => "T", 8 => "K", 9 => "E", 10 => "Ex", 11 => "C", _ => "M",
        }));
        if self.red_color {
          td.set_class_name("red");
        } else {
          td.set_class_name("");
        }
      }
    }
  }

  ///Returns false if not advancing paper.
  ///Returns true if advancing paper.
  pub fn new_advance_paper_signal(&mut self, ram0_data: u4) -> bool {
    let advance_paper = ram0_data.value() & 0b1000 == 0b1000;
    if !self.advance_paper && advance_paper { //We only signal on the switch from false to true.
      self.advance_paper = true;
      return true;
    }
    self.advance_paper = advance_paper;
    false
  }

  ///Returns false if not hammering.
  ///Returns true if hammering.
  pub fn new_hammer_signal(&mut self, ram0_data: u4) -> bool {
    let hammering = ram0_data.value() & 0b10 == 0b10;
    if !self.hammering && hammering {
      self.hammering = true;
      return true;
    }
    self.hammering = hammering;
    false
  }

  fn move_up_paper(&mut self) {
    for i in 0..6 {
      copy_tr(
        self.paper_rows.item(i).unwrap(),
        self.paper_rows.item(i + 1).unwrap()
      );
    }
    clear_tr(self.paper_rows.item(6).unwrap());
    self.red_color = false;
  }

  fn hit_hammer(&self, bits: u32) {
    let cells = self.paper_rows.item(6).expect("can't get tr").children();  //Lowest paper line
    self.write_digits(cells, bits);
  }
}

fn printer_shift_bits(shifters: &[shifter4003::Shifter]) -> u32 {
  let shift1 = shifters[1].read_parallel() as u32;
  let shift2 = shifters[2].read_parallel() as u32;
  shift1 | (shift2 << 10)
}

fn copy_tr(tr_to: web_sys::Element, tr_from: web_sys::Element) {
  let to_children = tr_to.children();
  let from_children = tr_from.children();
  for i in 0..to_children.length() {
    let to_child = to_children.item(i).unwrap();
    let from_child = from_children.item(i).unwrap();
    to_child.set_text_content(Some(&from_child.text_content().unwrap()));
    to_child.set_class_name(&from_child.class_name());
  }
}

fn clear_tr(tr: web_sys::Element) {
  let children = tr.children();
  for i in 0..children.length() {
    let child = children.item(i).unwrap();
    child.set_text_content(Some(" "));  //Leave a space to make a default size..
    child.set_class_name("");
  }
}