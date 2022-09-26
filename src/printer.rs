use arbitrary_int::u4;
use boards::busicom141pf::Board;

pub(super) struct Printer {
  spin_count: usize,  //There are a total of 13 rotations. However, we need to signal the processor that it was switched, on / off, so we have 26 rotations, switching on and off.
  red_color: bool,
  html_document: web_sys::Document,
}

impl Printer {
  pub(super) fn new() -> Self {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    Self {
      spin_count: 0,
      red_color: false,
      html_document: document,
    }
  }
  
  pub(super) fn run_sleep_cycle(&mut self, board: &mut Board) {
    if self.spin_count % 2 == 1 {
      board.i4004.set_test_flag(false);
    } else {
      board.i4004.set_test_flag(true);
      self.spin_printer();
      
      let mut rom_ports = board.i4001s[2].read_ports();
      if self.spin_count == 26 {
        rom_ports |= u4::new(0b1);  //Every full rotation of the printer, we set this to 1.
        self.spin_count = 0;
      } else {
        rom_ports &= u4::new(0b1110);
      }
      board.i4001s[2].write_ports(rom_ports);
    }
    self.spin_count += 1;
  }
  
  pub(super) fn run_cycle(&mut self, board: &mut Board) {
    if board.new_advance_paper_signal() {
      self.move_up_paper();
    }
    if board.new_hammer_signal() {
      self.hit_hammer(board.printer_shift_bits());
    }

    //Red color change is per page, not per character. Once set, it can't be changed until you go to the next line.
    let ram0 = board.i4002s[0].read_ports().value();
    if ram0 & 0b1 == 0b1 {
      self.red_color = true;
    }
  }

  fn spin_printer(&self) {
    let table = self.html_document.get_element_by_id("printer").unwrap();
    let tbody = table.children().item(0).expect("can't get tbody");

    let td_list = tbody.children().item(0).expect("can't get tr").children();
    self.write_digits(td_list, 0b11111111111111111111);
  }

  fn write_digits(&self, td_list: web_sys::HtmlCollection, bits: usize) {
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

  fn move_up_paper(&mut self) {
    let table = self.html_document.get_element_by_id("paper").unwrap();
    let tbody = table.children().item(0).expect("can't get tbody");
    let tbody_children = tbody.children();
    for i in 0..6 {
      copy_tr(
        tbody_children.item(i).unwrap(),
        tbody_children.item(i + 1).unwrap()
      );
    }
    clear_tr(tbody_children.item(6).unwrap());
    self.red_color = false;
  }

  fn hit_hammer(&self, bits: usize) {
    let table = self.html_document.get_element_by_id("paper").unwrap();
    let tbody = table.children().item(0).expect("can't get tbody");
    
    let td_list = tbody.children().item(6).expect("can't get tr").children();  //Lowest paper line

    self.write_digits(td_list, bits);
  }
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