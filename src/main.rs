fn main() {
    match ios::extractor() {
        Ok(res) => {
            println!("{}", res);
        }
        Err(err) => {
            println!("{}", err);
        }
    };
}
