//! A very small XML reader: just enough for Android `strings.xml` files.
//!
//! It understands elements, attributes, text, comments, processing instructions,
//! CDATA and the predefined and numeric character references.

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Element(Element),
    Text(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
    /// Line (starting at 1) where the element starts.
    pub line: usize,
}

impl Element {
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|c| match c {
            Node::Element(e) => Some(e),
            Node::Text(_) => None,
        })
    }

    /// The text directly inside the element, with CDATA and references decoded.
    pub fn text(&self) -> String {
        self.children
            .iter()
            .filter_map(|c| match c {
                Node::Text(t) => Some(t.as_str()),
                Node::Element(_) => None,
            })
            .collect()
    }
}

struct Reader<'a> {
    source: &'a str,
    position: usize,
}

pub fn parse(source: &str) -> Result<Element, String> {
    let mut reader = Reader {
        source,
        position: 0,
    };
    reader.skip_prolog()?;
    let root = reader.element()?;
    reader.skip_misc()?;
    if reader.position < source.len() {
        return Err(format!(
            "line {}: unexpected content after the root element",
            reader.line()
        ));
    }
    Ok(root)
}

impl Reader<'_> {
    fn rest(&self) -> &str {
        &self.source[self.position..]
    }

    fn line(&self) -> usize {
        self.source[..self.position].matches('\n').count() + 1
    }

    fn error<T>(&self, message: &str) -> Result<T, String> {
        Err(format!("line {}: {message}", self.line()))
    }

    fn skip_whitespace(&mut self) {
        let trimmed = self.rest().trim_start();
        self.position = self.source.len() - trimmed.len();
    }

    fn skip_past(&mut self, terminator: &str, what: &str) -> Result<(), String> {
        match self.rest().find(terminator) {
            Some(i) => {
                self.position += i + terminator.len();
                Ok(())
            }
            None => self.error(&format!("unterminated {what}")),
        }
    }

    /// Whitespace, comments and processing instructions.
    fn skip_misc(&mut self) -> Result<(), String> {
        loop {
            self.skip_whitespace();
            if self.rest().starts_with("<!--") {
                self.skip_past("-->", "comment")?;
            } else if self.rest().starts_with("<?") {
                self.skip_past("?>", "processing instruction")?;
            } else {
                return Ok(());
            }
        }
    }

    fn skip_prolog(&mut self) -> Result<(), String> {
        self.skip_misc()?;
        if self.rest().starts_with("<!DOCTYPE") {
            self.skip_past(">", "DOCTYPE")?;
            self.skip_misc()?;
        }
        Ok(())
    }

    fn name(&mut self) -> Result<String, String> {
        let end = self
            .rest()
            .find(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '-' | ':' | '.')))
            .unwrap_or(self.rest().len());
        if end == 0 {
            return self.error("expected a name");
        }
        let name = self.rest()[..end].to_string();
        self.position += end;
        Ok(name)
    }

    fn element(&mut self) -> Result<Element, String> {
        if !self.rest().starts_with('<') {
            return self.error("expected '<'");
        }
        let line = self.line();
        self.position += 1;
        let name = self.name()?;
        let mut attributes = Vec::new();
        loop {
            self.skip_whitespace();
            if self.rest().starts_with("/>") {
                self.position += 2;
                return Ok(Element {
                    name,
                    attributes,
                    children: Vec::new(),
                    line,
                });
            }
            if self.rest().starts_with('>') {
                self.position += 1;
                break;
            }
            let attribute = self.name()?;
            self.skip_whitespace();
            if !self.rest().starts_with('=') {
                return self.error("expected '=' after an attribute name");
            }
            self.position += 1;
            self.skip_whitespace();
            let quote = match self.rest().chars().next() {
                Some(q @ ('"' | '\'')) => q,
                _ => return self.error("expected a quoted attribute value"),
            };
            self.position += 1;
            let Some(end) = self.rest().find(quote) else {
                return self.error("unterminated attribute value");
            };
            let value = decode_references(&self.rest()[..end])
                .map_err(|e| format!("line {}: {e}", self.line()))?;
            self.position += end + 1;
            attributes.push((attribute, value));
        }

        let mut children = Vec::new();
        loop {
            let rest = self.rest();
            if rest.is_empty() {
                return Err(format!("line {line}: <{name}> is never closed"));
            } else if let Some(after) = rest.strip_prefix("</") {
                let end = after
                    .find('>')
                    .ok_or_else(|| format!("line {}: unterminated closing tag", self.line()))?;
                let closing = after[..end].trim();
                if closing != name {
                    return self.error(&format!(
                        "</{closing}> does not match <{name}> from line {line}"
                    ));
                }
                self.position += 2 + end + 1;
                return Ok(Element {
                    name,
                    attributes,
                    children,
                    line,
                });
            } else if rest.starts_with("<!--") {
                self.skip_past("-->", "comment")?;
            } else if let Some(after) = rest.strip_prefix("<![CDATA[") {
                let end = after
                    .find("]]>")
                    .ok_or_else(|| format!("line {}: unterminated CDATA", self.line()))?;
                children.push(Node::Text(after[..end].to_string()));
                self.position += "<![CDATA[".len() + end + 3;
            } else if rest.starts_with('<') {
                children.push(Node::Element(self.element()?));
            } else {
                let end = rest.find('<').unwrap_or(rest.len());
                let text = decode_references(&rest[..end])
                    .map_err(|e| format!("line {}: {e}", self.line()))?;
                children.push(Node::Text(text));
                self.position += end;
            }
        }
    }
}

fn decode_references(text: &str) -> Result<String, String> {
    if !text.contains('&') {
        return Ok(text.to_string());
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find(';').ok_or("unterminated character reference")?;
        let reference = &after[..end];
        let decoded = match reference {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                let number = reference
                    .strip_prefix('#')
                    .ok_or_else(|| format!("unknown entity &{reference};"))?;
                let code = match number.strip_prefix(['x', 'X']) {
                    Some(hex) => u32::from_str_radix(hex, 16),
                    None => number.parse(),
                }
                .map_err(|_| format!("invalid character reference &{reference};"))?;
                char::from_u32(code)
                    .ok_or_else(|| format!("invalid character reference &{reference};"))?
            }
        };
        out.push(decoded);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_elements_attributes_and_text() {
        let root = parse(
            "<?xml version=\"1.0\"?>\n<!-- c -->\n<resources a='1'>\n  <string name=\"x\">Hi &amp; bye</string>\n  <empty/>\n</resources>\n",
        )
        .unwrap();
        assert_eq!(root.name, "resources");
        assert_eq!(root.attribute("a"), Some("1"));
        let children: Vec<_> = root.elements().collect();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].attribute("name"), Some("x"));
        assert_eq!(children[0].text(), "Hi & bye");
        assert_eq!(children[0].line, 4);
        assert!(children[1].children.is_empty());
    }

    #[test]
    fn decodes_references_and_cdata() {
        let root = parse("<r><s>&lt;b&gt; &#65;&#x42; <![CDATA[<raw & text>]]></s></r>").unwrap();
        assert_eq!(
            root.elements().next().unwrap().text(),
            "<b> AB <raw & text>"
        );
    }

    #[test]
    fn rejects_malformed_documents() {
        for source in [
            "",
            "<a>",
            "<a></b>",
            "<a><b></a></b>",
            "<a x=1></a>",
            "<a x=\"1></a>",
            "<a>&nope;</a>",
            "<a>&amp</a>",
            "<a><!-- never closed </a>",
            "<a></a>trailing",
            "<a></a><b/>",
        ] {
            assert!(parse(source).is_err(), "{source:?} should not parse");
        }
    }

    #[test]
    fn errors_carry_a_line_number() {
        let error = parse("<a>\n<b>\n</a>").unwrap_err();
        assert!(error.starts_with("line 3:"), "{error}");
    }
}
