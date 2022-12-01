#[cfg(not(target_arch = "wasm32"))]
mod target_non_wasm32 {
    use std::collections::HashMap;
    use std::fmt::Write;
    use std::path::PathBuf;

    use bounce::helmet::{self, HelmetTag};
    use clap::Parser;
    use helmet_ssr::{ServerApp, ServerAppProps};
    use warp::path::FullPath;
    use warp::Filter;

    /// A basic example
    #[derive(Parser, Debug)]
    struct Opt {
        /// the "dist" created by trunk directory to be served for hydration.
        #[clap(short, long)]
        dir: PathBuf,
    }

    async fn render(
        script_content: String,
        url: String,
        queries: HashMap<String, String>,
    ) -> String {
        let (renderer, writer) = helmet::render_static();

        let body_s = yew::ServerRenderer::<ServerApp>::with_props(move || ServerAppProps {
            url: url.into(),
            queries,
            helmet_writer: writer,
        })
        .render()
        .await;

        let mut html_tag: Option<HelmetTag> = None;
        let mut body_tag: Option<HelmetTag> = None;

        let mut helmet_s = "".to_string();

        let rendered: Vec<HelmetTag> = renderer.render().await;
        let mut s = String::with_capacity(body_s.len());

        for tag in rendered {
            let _ = tag.write_static(&mut helmet_s);

            match tag {
                HelmetTag::Html { .. } => {
                    html_tag = Some(tag);
                }
                HelmetTag::Body { .. } => {
                    body_tag = Some(tag);
                }
                _ => {}
            }
        }

        let _ = writeln!(s, "<!doctype html>");
        {
            let mut html_s = String::new();
            html_tag.map(|m| m.write_attrs(&mut html_s));

            if html_s.is_empty() {
                let _ = writeln!(s, "<html>");
            } else {
                let _ = writeln!(s, "<html {}>", html_s);
            }
        }
        let _ = writeln!(s, "<head>");
        s.push_str(&helmet_s);
        let _ = writeln!(s, r#"<script type="module">{}</script>"#, script_content);
        let _ = writeln!(s, "</head>");

        {
            let mut body_s = String::new();
            body_tag.map(|m| m.write_attrs(&mut body_s));

            if body_s.is_empty() {
                let _ = writeln!(s, "<body>");
            } else {
                let _ = writeln!(s, "<body {}>", body_s);
            }
        }
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

        let render_f = warp::path::full().and(warp::query()).then(
            move |path: FullPath, queries: HashMap<String, String>| {
                let script_content = script_content.clone();

                async move {
                    warp::reply::html(
                        render(script_content, path.as_str().to_string(), queries).await,
                    )
                }
            },
        );

        let routes = warp::path::end()
            .and(render_f.clone())
            .or(warp::path("index.html").and(render_f.clone()))
            .or(warp::fs::dir(opts.dir.clone()))
            .or(render_f);

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
