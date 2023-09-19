mod api;
use api::Api;

pub use anyhow::Result;

// std
use std::env;
// crates.io
use actix_web::{App, HttpResponse, HttpServer, Responder};
use futures::future;
use once_cell::sync::Lazy;

static API: Lazy<Api> = Lazy::new(Api::new);

#[actix_web::get("/")]
async fn query() -> impl Responder {
	let mut data = String::new();

	if let Ok(vs) = API.collators().await {
		future::join_all(vs.into_iter().map(|v| async move {
			let commission_history = loop {
				if let Ok(c) = API.commission_history_of(&v).await {
					break c;
				} else {
					continue;
				}
			};

			format!(
				r#"<tr><td><a href="https://darwinia.subscan.io/account/{v}">{v}</td><td>{}</td><td>{}</td></tr>"#,
				commission_history.commissions(),
				commission_history.reputation()
			)
		}))
		.await
		.into_iter()
		.for_each(|d| data.push_str(&d));
	}

	let body = format!(
		r#"<!DOCTYPE html>
        <html>
          <head>
            <title>Commission Board</title>
            <style>
              html, body {{
                height: 100%;
              }}

              body {{
                display: flex;
                align-items: center;
                justify-content: center;
                background-color: #1c2331;
                color: #fff;
                font-family: monospace;
				font-size: 18px;
              }}

              th:nth-child(1), td:nth-child(1) {{
                background-color: #2c3e50;
              }}

              th:nth-child(2), td:nth-child(2) {{
                background-color: #2980b9;
              }}

              th:nth-child(3), td:nth-child(3) {{
                background-color: #27ae60;
              }}

              th, td {{
                text-align: center;
              }}

              a {{
                color: white;
              }}
            </style>
          </head>
          <body>
            <table style="margin: 0 auto;">
              <tr>
                <th>Collator</th>
                <th>Commission History (Block,Value)</th>
                <th>Reputation Base on Recent 5 Changes</th>
              </tr>
              {data}
            </table>
          </body>
        </html>"#,
	);

	HttpResponse::Ok().body(body)
}

#[actix_web::main]
async fn main() -> Result<()> {
	HttpServer::new(|| App::new().service(query))
		.bind(("0.0.0.0", env::var("COMMISSION_BOARD_PORT")?.parse()?))?
		.run()
		.await?;

	Ok(())
}
