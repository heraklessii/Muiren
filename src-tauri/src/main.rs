// Windows'ta surum derlemesinde arkada konsol penceresi acilmasin.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    muiren_lib::run()
}
