use proc_macro2::LineColumn;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    Expr, File, Macro,
};

use super::format::HtmlMacro;


#[derive(Default)]
struct DomMacroVisitor<'ast> {
    indent_stack: Vec<LineColumn>,
    macros: Vec<HtmlMacro<'ast>>,
}

impl<'ast> Visit<'ast> for DomMacroVisitor<'ast> {
    fn visit_stmt(&mut self, i: &'ast syn::Stmt) {
        self.indent_stack.push(i.span().start());
        visit::visit_stmt(self, i);
        self.indent_stack.pop();
    }

    fn visit_expr(&mut self, i: &'ast Expr) {
        self.indent_stack.push(i.span().start());
        visit::visit_expr(self, i);
        self.indent_stack.pop();
    }

    fn visit_arm(&mut self, i: &'ast syn::Arm) {
        self.indent_stack.push(i.span().start());
        visit::visit_arm(self, i);
        self.indent_stack.pop();
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        if node.path.is_ident("html") {
            let span_start = node.span().start().column;
            let span_line = node.span().start().line;
            let indent = self
                .indent_stack
                .iter()
                .filter(|v| v.line == span_line && v.column < span_start)
                .map(|v| v.column)
                .min()
                .unwrap_or(
                    self.indent_stack
                        .iter()
                        .rev()
                        .find(|v| v.column < span_start)
                        .map(|i| i.column)
                        .unwrap_or(0),
                );

            if let Some(dom_mac) = HtmlMacro::try_parse(Some(indent), node) {
                self.macros.push(dom_mac);
            }
        }

        visit::visit_macro(self, node);
    }
}

pub fn collect_macros_in_file(file: &File) -> Vec<HtmlMacro> {
    let mut visitor = DomMacroVisitor::default();
    visitor.visit_file(file);
    visitor.macros
}