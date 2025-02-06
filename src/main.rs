fn main() {
    match ios::retriever() {
        Ok(res) => {
            println!("{}", res);
        }
        Err(err) => {
            println!("{}", err);
        }
    };
}
