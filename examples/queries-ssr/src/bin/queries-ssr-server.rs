#[cfg(not(target_arch = "wasm32"))]
mod target_non_wasm32 {
    use std::fmt::Write;
    use std::path::PathBuf;

    use clap::Parser;
    use queries_ssr::App;
    use warp::Filter;

    /// A basic example
    #[derive(Parser, Debug)]
    struct Opt {
        /// the "dist" created by trunk directory to be served for hydration.
        #[clap(short, long)]
        dir: PathBuf,
    }

    async fn render(script_content: String) -> String {
        let body_s = yew::ServerRenderer::<App>::new().render().await;

        let mut s = String::new();

        let _ = writeln!(s, "<!doctype html>");
        let _ = writeln!(s, "<html>");
        let _ = writeln!(s, "<head>");

        let _ = writeln!(s, r#"<script type="module">{}</script>"#, script_content);
        let _ = writeln!(s, "</head>");

        let _ = writeln!(s, "<body>");
        s.push_str(&body_s);

        let _ = writeln!(s, "</body>");
        let _ = writeln!(s, "</html>");

        s
    }

    fn extract_script_content(index_html: &str) -> String {
        fn extract<I>(nodes: I) -> Option<String>
        where
            I: IntoIterator<Item = html_parser::Node>,
        {
            fn extract_text<I>(nodes: I) -> Option<String>
            where
                I: IntoIterator<Item = html_parser::Node>,
            {
                for node in nodes {
                    match node {
                        html_parser::Node::Comment(_) => {}
                        html_parser::Node::Element(_) => {}
                        html_parser::Node::Text(t) => {
                            return Some(t);
                        }
                    }
                }

                None
            }

            let nodes = nodes.into_iter();
            for node in nodes {
                match node {
                    html_parser::Node::Comment(_) => {}
                    html_parser::Node::Element(e) => {
                        if e.name.to_lowercase().as_str() == "script" {
                            return extract_text(e.children);
                        }

                        if let Some(m) = extract(e.children) {
                            return Some(m);
                        }
                    }
                    html_parser::Node::Text(_) => {}
                }
            }

            None
        }

        let dom = html_parser::Dom::parse(index_html).expect("failed to parse");

        extract(dom.children).expect("failed to find script tag")
    }

    pub async fn main() {
        env_logger::init();

        let opts = Opt::parse();

        let index_html_s = tokio::fs::read_to_string(opts.dir.join("index.html"))
            .await
            .expect("failed to read index.html");

        let script_content = extract_script_content(&index_html_s);

        let render_f = warp::get().then(move || {
            let script_content = script_content.clone();

            async move { warp::reply::html(render(script_content).await) }
        });

        let routes = warp::path::end()
            .and(render_f.clone())
            .or(warp::path("index.html").and(render_f.clone()))
            .or(warp::fs::dir(opts.dir.clone()));

        println!("You can view the website at: http://localhost:8080/");
        warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    target_non_wasm32::main().await;
}

#[cfg(target_arch = "wasm32")]
fn main() {}
