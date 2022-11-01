//! On veut accéder à un serveur web exposé uniquement en 127.0.0.1
//! L'attaquant sera le client du proxy et la cible le serveur.
//! 
//! Attaquant (client) <--------------------> Serveur (proxy) <-----------------------> localhost (cible)
//!                      Connection entrante                      Connection sortante
//!                         = Listener                                  = out_conn
//!                    -----client_read----->                     -----server_write----->
//!                    <------client_write---                     <----server_read------


use tokio::{net::{TcpListener, TcpStream}, try_join};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    // Accepte les connections entrantes et crée un thread
    while let Ok((client_conn,_)) = listener.accept().await {
        tokio::spawn(async move {
            println!("Connection établie");
            process_socket(client_conn).await;
        });
    }

    Ok(())
}

/// Crée une connection avec la cible (127.0.0.1:port)
/// Split la connection entrante en ReadHalf et WriteHalf
/// Copie le flux de la connection entrant vers la connection sortante
async fn process_socket(mut client_conn: TcpStream) -> Result<(), Box<dyn std::error::Error>>{
    let mut out_conn = TcpStream::connect("127.0.0.1:7331").await?;

    let (mut client_read, mut client_write) = client_conn.split();
    let (mut server_read, mut server_write) = out_conn.split();

    let in_conn = tokio::io::copy(&mut client_read, &mut server_write);
    let out_conn = tokio::io::copy(&mut server_read, &mut client_write);

    try_join!(in_conn, out_conn)?;

    Ok(())
}