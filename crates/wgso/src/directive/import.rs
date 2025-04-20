use crate::directive::tokens::{Ident, Lexer, Token, TokenKind};
use crate::Error;
use std::ops::Range;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct ImportDirective {
    pub(crate) path: Vec<Ident>,
    pub(crate) span: Range<usize>,
}

impl ImportDirective {
    pub(crate) fn parse(lexer: &mut Lexer<'_>, hashtag: &Token<'_>) -> Result<Self, Error> {
        let mut path = vec![Ident::parse(lexer)?];
        while lexer.clone().next_expected(&[TokenKind::Dot]).is_ok() {
            lexer.next_expected(&[TokenKind::Dot])?;
            path.push(Ident::parse(lexer)?);
        }
        Ok(Self {
            path,
            span: hashtag.span.start..lexer.offset(),
        })
    }

    pub(crate) fn file_path(&self, root_path: &Path) -> PathBuf {
        root_path.join(
            self.path
                .iter()
                .enumerate()
                .map(|(index, segment)| {
                    if index == self.path.len() - 1 {
                        format!("{}.wgsl", segment.label.clone())
                    } else {
                        segment.label.clone()
                    }
                })
                .collect::<PathBuf>(),
        )
    }
}
