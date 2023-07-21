mod analyzer;
pub mod llvm;
pub mod parser;
mod typechecker;
use std::ops::Range;

use llvm::CodeGen;

use crate::parser::*;
use chumsky::{input::Stream, prelude::*};
use logos::Logos;
use miette::{miette, LabeledSpan};

fn main() {
    const SRC: &str = r#"
let func() -> int {
    let i: int = 4
    return i
}
"#;

    let token_iter = LogosToken::lexer(SRC)
        .spanned()
        .map(|(tok, span)| match tok {
            Ok(tok) => (tok, span.into()),
            Err(()) => (LogosToken::Error, span.into()),
        });

    let token_stream = Stream::from_iter(token_iter)
        .spanned::<LogosToken, SimpleSpan>((SRC.len()..SRC.len()).into());
    match parser().parse(token_stream).into_result() {
        Ok(o) => {
            let mut analyzer = analyzer::Analyzer::new(o.clone(), SRC);
            if analyzer.analyze() {
                let mut checker = typechecker::TypeChecker::new(o.clone(), SRC);
                checker.typecheck();

                let context = inkwell::context::Context::create();
                let mut code_gen = CodeGen::new(&context);
                code_gen.jit_run(&o);
            }
        }
        Err(errs) => {
            for err in errs {
                let span: Range<usize> = (*err.span()).into();
                let reason = err.reason().to_string();
                println!(
                    "{:?}",
                    miette!(
                        labels = vec![LabeledSpan::at(span, reason)],
                        "Parsing error"
                    )
                    .with_source_code(SRC)
                );
            }
        }
    };
}
