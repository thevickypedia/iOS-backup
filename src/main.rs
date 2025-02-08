// todo:
//  Next major release: Support multiple serial numbers as a comma separated list
//  Include an option to extract ALL backups
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
