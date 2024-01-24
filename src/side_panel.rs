use arbitrary_int::u2;
use chips::mcs4;
use chips::{Indexer16, Indexer64};

pub struct SidePanel {
  memory_tr: web_sys::HtmlCollection,
  character_data: [[Indexer64; 4];2],
  status_data: [[Indexer16; 4]; 2],
  ports_tr: web_sys::HtmlCollection,
  rom_ports: [u8; 5],
  ram_ports: [u8; 2],
  shifts_tr: web_sys::HtmlCollection,
  shift_data: [u16; 3],
}

impl SidePanel {
  
  pub fn new() -> Self {
    Self {
      memory_tr: get_tr_list("ram"),
      character_data: Default::default(),
      status_data: Default::default(),
      ports_tr: get_tr_list("ports"),
      rom_ports: [0; 5],
      ram_ports: [0; 2],
      shifts_tr: get_tr_list("shifts"),
      shift_data: [0; 3],
    }
  }

  pub(super) fn print_memory(&mut self, board: &mcs4::Board) {
    let tr_list = &self.memory_tr;

    for ram in 0..=1 {
      for register_index in 0..=3_usize {
        let html_index = (register_index as u32) + 2 + (ram as u32) * 7;
        let td_list = tr_list.item(html_index).expect("can't get tr").children();
        
        let character_data = board.rams[ram].read_full_character(u2::new(register_index as u8));
        let status_data = board.rams[ram].read_full_status(u2::new(register_index as u8));
        
        if self.character_data[ram][register_index] != character_data || self.status_data[ram][register_index] != status_data {
          self.character_data[ram][register_index] = character_data;
          self.status_data[ram][register_index] = status_data;
          print_register(
            &td_list,
            character_data,
            status_data
          );
        }
      }
    }
  }

  pub(super) fn print_ports(&mut self, board: &mcs4::Board) {
    let tr_list = &self.ports_tr;

    let mut html_index = 1;
    for index in 0..=4 { //ROMS
      let ports = board.roms[index].ports.value();
      if self.rom_ports[index] != ports {
        self.rom_ports[index] = ports;
        let td_list = tr_list.item(html_index).expect("can't get tr").children();
        print_line_ports(&td_list, ports);
      }
      html_index += 1;
    }
    for index in 0..=1 { //RAMS
      let ports = board.rams[index].ports.value();
      if self.ram_ports[index] != ports {
        self.ram_ports[index] = ports;
        let td_list = tr_list.item(html_index).expect("can't get tr").children();
        print_line_ports(&td_list, ports);
      }
      html_index += 1;
    }
  }
  
  pub(super) fn print_shifts(&mut self, shifters: &[mcs4::shifter4003::Shifter]) {
    let tr_list = &self.shifts_tr;

    let mut html_index = 1;
    for index in 0..=2 {
      let bits = shifters[index].read_parallel();
      if self.shift_data[index] != bits {
        self.shift_data[index] = bits;
        let td_list = tr_list.item(html_index).expect("can't get tr").children();
        print_line_shifts(&td_list, bits);
      }
      html_index += 1;
    }
  }
}

fn print_register(td_list: &web_sys::HtmlCollection, chars: Indexer64, statuses: Indexer16) {
  let mut html_index = 2;
  for character_index in (0..16).rev() {
    if let Some(td) = td_list.item(html_index) {
      td.set_text_content(Some(&format!("{:X}", chars.read_nibble(character_index).value())));
    }
    html_index += 1;
  }
  html_index = 19;
  for status_index in (0..4).rev() {
    if let Some(td) = td_list.item(html_index) {
      td.set_text_content(Some(&format!("{:X}", statuses.read_nibble(status_index).value())));
    }
    html_index += 1;
  }
}

fn print_line_ports(td_list: &web_sys::HtmlCollection, mut ports: u8) {
  for html_index in 1..=4 {
    if let Some(td) = td_list.item(html_index) {
      if ports & 1 == 1 {
        td.set_text_content(Some("●"));
      } else {
        td.set_text_content(Some("○"));
      }
      ports >>= 1;
    }
  }
}

fn print_line_shifts(td_list: &web_sys::HtmlCollection, bits: u16) {
  for index in 0..10 {
    let html_index = (index + 1) as u32;
    if let Some(td) = td_list.item(html_index) {
      if (bits >> index) & 1 == 0 {
        td.set_text_content(Some("○"));
      } else {
        td.set_text_content(Some("●"));
      }
    }
  }
}

fn get_tr_list(table_id: &str) -> web_sys::HtmlCollection {
  let window = web_sys::window().expect("no global `window` exists");
  let document = window.document().expect("should have a document on window");
  let table = document.get_element_by_id(table_id).expect("can't find ram id");
  let tbody = table.children().item(0).expect("can't get tbody");
  tbody.children()
}