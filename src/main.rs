use html5gum::{Token, Tokenizer};
use regex::{Captures, Regex};

use std::{borrow::Cow, fmt::Write};

fn modify_xml(html: &str) -> String {
    let ignore_tags = ["style", "form", "script"];
    let mut new_html = String::new();

    let mut ignore = false;

    for Ok(token) in Tokenizer::new(html) {
        match token {
            Token::StartTag(tag) => {
                if ignore {
                    continue;
                }
                let t = String::from_utf8_lossy(&tag.name).to_string();

                if ignore_tags.iter().any(|e| e == &t) {
                    // ignore
                    ignore = true;
                } else {
                    let mut attr = String::new();
                    for (key, value) in &tag.attributes {
                        write!(
                            attr,
                            " {}=\"{}\"",
                            String::from_utf8_lossy(key),
                            String::from_utf8_lossy(value)
                        )
                        .unwrap();
                    }

                    write!(
                        new_html,
                        "<{}{}>",
                        String::from_utf8_lossy(&tag.name),
                        &attr.trim_end()
                    )
                    .unwrap();
                }
            }
            Token::String(hello_world) => {
                if ignore {
                    continue;
                }
                write!(new_html, "{}", unsafe {
                    String::from_utf8_unchecked(hello_world.to_vec())
                })
                .unwrap();
            }
            Token::EndTag(tag) => {
                let t = String::from_utf8_lossy(&tag.name).to_string();
                if ignore_tags.iter().any(|e| e == &t) {
                    // ignore
                    if ignore {
                        // End ignore
                        ignore = false;
                    }
                } else {
                    write!(new_html, "</{}>", String::from_utf8_lossy(&tag.name)).unwrap();
                }
            }
            Token::Error(err) => {
                println!("Error: {}", err)
            }
            Token::Comment(s) => {
                println!("{:?}", s)
            }
            Token::Doctype(s) => {
                println!("{:?}", s)
            } // _ => {
              //     println!("unexpected input {:?}", token);

              //     // panic!("unexpected input")
              // }
        }
    }

    new_html
}

fn clean_head(target: &[u8]) -> Cow<'_, [u8]> {
    let re = regex::bytes::Regex::new(r"(?s)<head>.*</head>").unwrap();
    let head = re.find(target).unwrap().as_bytes();

    let head = remove_script(head);

    let head = remove_style(&head);

    let head = remove_meta(&head);
    let head = remove_form(&head);
    println!("Head Source: {}", String::from_utf8_lossy(&head));

    let html = re.replace_all(target, head);

    return html;
}

fn clean_body(target: &[u8]) -> Vec<u8> {
    let re_start = regex::bytes::Regex::new(r"<body>").unwrap();
    let re_end = regex::bytes::Regex::new(r"</body>").unwrap();

    let start = re_start.find(&target).unwrap().start();
    let end = re_end.find(&target).unwrap().end();

    let body = &target[start..end];

    let body = remove_script(body);
    let body = remove_style(&body);
    let body = remove_form(&body);
    let body = remove_trees(&body);
    let body = remove_empty_lines(&body);
    // let body = remove_meta(&body);

    let html = [&target[..start], &body, &target[end..]].concat();
    return html;
}

fn remove_empty_lines(target: &[u8]) -> Cow<'_, [u8]> {
    let re = regex::bytes::Regex::new(r"[\r\n]*").unwrap();

    let replacement = re.replace_all(target, b"");
    return replacement;
}

fn remove_trees(target: &[u8]) -> Cow<'_, [u8]> {
    let re = regex::bytes::Regex::new(
        r#"<p id="gs_dont_print">Save trees - read court opinions online on Google Scholar.</p>"#,
    )
    .unwrap();

    let replacement = re.replace_all(target, b"");
    return replacement;
}

fn remove_form(target: &[u8]) -> Cow<'_, [u8]> {
    let re = regex::bytes::Regex::new(r"(?s)<form.*>.*</form>").unwrap();
    let replacement = re.replace_all(target, b"");
    return replacement;
}

fn remove_script(target: &[u8]) -> Cow<'_, [u8]> {
    let re = regex::bytes::Regex::new(r"(?s)<script.*>.*</script>").unwrap();
    let replacement = re.replace_all(target, b"");
    return replacement;
}

fn remove_style(target: &[u8]) -> Cow<'_, [u8]> {
    let re = regex::bytes::Regex::new(r"(?s)<style.*>.*</style>").unwrap();
    let replacement = re.replace_all(target, b"");
    return replacement;
}

fn remove_meta(target: &[u8]) -> Cow<'_, [u8]> {
    let re = regex::bytes::Regex::new(r"(?s)<meta[^>]*>").unwrap();
    let replacement = re.replace_all(target, b"");
    return replacement;
}

async fn download_case(target: &str) -> Result<Vec<u8>, ()> {
    let body = reqwest::get(target).await;

    match body {
        Ok(r) => {
            let b = r.bytes().await;
            match b {
                Ok(bytes) => {
                    println!("Download succeeded.");
                    Ok(bytes.to_vec())
                }
                Err(err) => Err(eprintln!("Download failed: {:?}", err)),
            }
        }
        Err(_err) => Err(()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let case = download_case("").await.unwrap();
    let s = unsafe { String::from_utf8_unchecked(case) };
    // let s = String::from_utf8_lossy(&case);
    let result = modify_xml(&s);
    // println!("{}", String::from_utf8_lossy(&result));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Write, prelude::*};

    #[tokio::test]
    async fn test_download_case() {
        let target = "https://scholar.google.com/scholar_case?case=16972936278337646094";
        let r = download_case(target).await;

        match r {
            Ok(html) => {
                println!("{}", String::from_utf8_lossy(&html))
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }

        // assert_eq!((), response);
    }

    #[tokio::test]
    async fn test_section() {
        let r = "D.C.Code § 23-110 (2001), con";
        println!("{:?}", r.as_bytes());
        let n = r.as_bytes();
        let s = String::from_utf8_lossy(&n);
        println!("{}", s);

        // assert_eq!((), response);
    }

    #[tokio::test]
    async fn test_modify() {
        let target = "https://scholar.google.com/scholar_case?case=16972936278337646094";
        let r = download_case(target).await;

        match r {
            Ok(html) => {
                let mut file = File::create("downloaded.html").unwrap();
                file.write_all(&html).unwrap();

                // let r = unsafe { String::from_utf8_unchecked(html) };
                //

                let mut head = clean_head(&html);
                // output
                let output = clean_body(&head);

                // let output = modify_xml(&String::from_utf8_lossy(&output));

                let mut file = File::create("output.html").unwrap();
                let r = file.write_all(&output);

                // let body = clean_body(&html);

                // combine
                // head.extend_from_slice(&body);

                // println!("CLEAN: {:?}", head);

                // let mut file = File::create("clean.html").unwrap();
                // let r = file.write_all(&head);

                // let r = remove_style(&r);

                // let r = remove_script(&r);
                // let r = remove_meta(&r);

                // let result = modify_xml(&r);

                // let mut file = File::create("test.html").unwrap();
                // file.write_all(r.as_bytes()).unwrap();

                // println!("{}", result);
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }

        // assert_eq!((), response);
    }

    #[tokio::test]
    async fn test_clean_head() {
        let data = std::fs::read("downloaded.html").unwrap();
        let head = clean_head(&data);
        let mut file = File::create("cleaned_head.html").unwrap();
        let r = file.write_all(&head);

        // assert_eq!((), response);
    }

    #[tokio::test]
    async fn test_clean_body() {
        let data = std::fs::read("downloaded.html").unwrap();

        // let data = b"<head><body>data</body>".to_vec();
        let body = clean_body(&data);
        let mut file = File::create("cleaned_body.html").unwrap();
        let r = file.write_all(&body);

        // assert_eq!((), response);
    }

    #[tokio::test]
    async fn test_finds_body() {
        let data = std::fs::read("downloaded.html").unwrap();

        let re_start = regex::bytes::Regex::new(r"<body>").unwrap();
        let re_end = regex::bytes::Regex::new(r"</body>").unwrap();

        let start = re_start.find(&data).unwrap().start();
        let end = re_end.find(&data).unwrap().end();

        let body = &data[start..end];

        // assert_eq!((), response);
    }

    #[tokio::test]
    async fn test_main() {
        // assert_eq!((), response);
        let r = main();
    }
}
