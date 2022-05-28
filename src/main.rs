use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::ReadHalf, TcpListener},
    sync::broadcast,
};
//Macro from tokio that wraps main() to an equivalent async version
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            //Allows you to "split" a socket so there is a read and write part and can handle them independently
            let (read, mut write) = socket.split();
            let mut reader: BufReader<ReadHalf> = BufReader::new(read);
            let mut line: String = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }

                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr)  = result.unwrap();

                        if addr != other_addr {
                            write.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
