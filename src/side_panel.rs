use arbitrary_int::u2;
use boards::busicom141pf::Board;
use chips::{Indexer16, Indexer64};

pub(super) fn print_memory(board: &Board) {
  let tr_list = get_tr_list("ram");

  for ram in 0..=1 {
    for register_index in 0..=3 {
      let html_index = (register_index as u32) + 2 + (ram as u32) * 7;
      let td_list = tr_list.item(html_index).expect("can't get tr").children();
      print_register(
        &td_list,
        board.i4002s[ram].read_full_character(u2::new(register_index)),
        board.i4002s[ram].read_full_status(u2::new(register_index))
      );
    }
  }
}

fn print_register(td_list: &web_sys::HtmlCollection, chars: Indexer64, statuses: Indexer16) {
  for character_index in 0..16 {
    let html_index = character_index as u32 + 2;
    if let Some(td) = td_list.item(html_index) {
      td.set_text_content(Some(&format!("{:X}", chars.read_nibble(character_index).value())));
    }
  }
  for status_index in 0..4 {
    let html_index = status_index as u32 + 19;
    if let Some(td) = td_list.item(html_index) {
      td.set_text_content(Some(&format!("{:X}", statuses.read_nibble(status_index).value())));
    }
  }
}

pub(super) fn print_ports(board: &Board) {
  let tr_list = get_tr_list("ports");

  let mut html_index = 1;
  for index in 0..=4 { //ROMS
    let ports = board.i4001s[index].read_ports().value();
    let td_list = tr_list.item(html_index).expect("can't get tr").children();
    print_line_ports(&td_list, ports);
    html_index += 1;
  }
  for index in 0..=1 { //RAMS
    let ports = board.i4002s[index].read_ports().value();
    let td_list = tr_list.item(html_index).expect("can't get tr").children();
    print_line_ports(&td_list, ports);
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


pub(super) fn print_shifts(board: &Board) {
  let tr_list = get_tr_list("shifts");

  let mut html_index = 1;
  for index in 0..=2 {
    let bits = board.i4003s[index].read_parallel();
    let td_list = tr_list.item(html_index).expect("can't get tr").children();
    print_line_shifts(&td_list, bits);
    html_index += 1;
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