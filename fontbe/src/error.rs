use std::{fmt::Display, io, path::PathBuf};

use fea_rs::compile::{error::CompilerError, PreviouslyAssignedClass};
use font_types::Tag;
use fontdrasil::types::GlyphName;
use fontir::{
    error::VariationModelError, orchestration::WorkId as FeWorkId, variations::DeltaError,
};
use read_fonts::ReadError;
use thiserror::Error;
use write_fonts::tables::{
    glyf::MalformedPath,
    gvar::{iup::IupError, GvarInputError},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO failure")]
    IoError(#[from] io::Error),
    #[error("Fea compilation failure {0}")]
    FeaCompileError(#[from] CompilerError),
    #[error("'{0}' {1}")]
    GlyphError(GlyphName, GlyphProblem),
    #[error("'{glyph_name}' {kurbo_problem:?} {context}")]
    KurboError {
        glyph_name: GlyphName,
        kurbo_problem: MalformedPath,
        context: String,
    },
    #[error("'{glyph}' references {referenced_glyph}, {problem}")]
    ComponentError {
        glyph: GlyphName,
        referenced_glyph: GlyphName,
        problem: GlyphProblem,
    },
    #[error("'{glyph}' {errors:?}")]
    ComponentErrors {
        glyph: GlyphName,
        errors: Vec<Error>,
    },
    #[error("Generating bytes for {context} failed: {e}")]
    DumpTableError {
        e: write_fonts::error::Error,
        context: String,
    },
    #[error("{what} out of bounds: {value}")]
    OutOfBounds { what: String, value: String },
    #[error("Unable to compute deltas for {0}: {1}")]
    GlyphDeltaError(GlyphName, DeltaError),
    #[error("Unable to assemble gvar")]
    GvarError(#[from] GvarInputError),
    #[error("Unable to read")]
    ReadFontsReadError(#[from] ReadError),
    #[error("IUP error for {0}: {1:?}")]
    IupError(GlyphName, IupError),
    #[error("Unable to interpret bytes as {0}")]
    InvalidTableBytes(Tag),
    #[error("Missing directory:{0}")]
    MissingDirectory(PathBuf),
    #[error("Variation model error in '{0}': {1}")]
    VariationModelError(GlyphName, VariationModelError),
    #[error("Missing file:{0}")]
    FileExpected(PathBuf),
    #[error("Missing {0}")]
    MissingTable(Tag),
    #[error("Expected an anchor, got {0:?}")]
    ExpectedAnchor(FeWorkId),
    #[error("No glyph id for '{0}'")]
    MissingGlyphId(GlyphName),
    #[error("No glyph class '{0}'")]
    MissingGlyphClass(GlyphName),
    #[error("Multiple assignments for class: {0:?}")]
    PreviouslyAssignedClass(PreviouslyAssignedClass),
}

#[derive(Debug)]
pub enum GlyphProblem {
    InconsistentComponents,
    InconsistentPathElements,
    HasComponentsAndPath,
    MissingDefault,
    NoComponents,
    NotInGlyphOrder,
}

impl Display for GlyphProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            GlyphProblem::HasComponentsAndPath => "has components *and* paths",
            GlyphProblem::InconsistentComponents => {
                "has different components at different points in designspace"
            }
            GlyphProblem::InconsistentPathElements => "has interpolation-incompatible paths",
            GlyphProblem::MissingDefault => "has no default master",
            GlyphProblem::NoComponents => "has no components",
            GlyphProblem::NotInGlyphOrder => "has no entry in glyph order",
        };
        f.write_str(message)
    }
}
