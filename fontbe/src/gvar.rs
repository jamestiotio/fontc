//! Generates a [gvar](https://learn.microsoft.com/en-us/typography/opentype/spec/gvar) table.

use std::sync::Arc;

use font_types::GlyphId;
use fontdrasil::{
    orchestration::{Access, Work},
    types::GlyphName,
};
use fontir::{ir::GlyphOrder, orchestration::WorkId as FeWorkId};
use write_fonts::{
    dump_table,
    tables::gvar::{GlyphDeltas, GlyphVariations, Gvar},
};

use crate::{
    error::Error,
    orchestration::{AnyWorkId, BeWork, Context, WorkId},
};

#[derive(Debug)]
struct GvarWork {}

pub fn create_gvar_work() -> Box<BeWork> {
    Box::new(GvarWork {})
}

fn make_variations(
    glyph_order: &GlyphOrder,
    get_deltas: impl Fn(&GlyphName) -> Vec<GlyphDeltas>,
) -> Vec<GlyphVariations> {
    glyph_order
        .iter()
        .enumerate()
        .filter_map(|(gid, gn)| {
            let deltas = get_deltas(gn);
            if deltas.is_empty() {
                return None;
            }
            Some((GlyphId::new(gid as u16), deltas))
        })
        .map(|(gid, deltas)| GlyphVariations::new(gid, deltas))
        .collect()
}

impl Work<Context, AnyWorkId, Error> for GvarWork {
    fn id(&self) -> AnyWorkId {
        WorkId::Gvar.into()
    }

    fn read_access(&self) -> Access<AnyWorkId> {
        Access::Custom(Arc::new(|id| {
            matches!(
                id,
                AnyWorkId::Fe(FeWorkId::GlyphOrder) | AnyWorkId::Be(WorkId::GvarFragment(..))
            )
        }))
    }

    /// Generate [gvar](https://learn.microsoft.com/en-us/typography/opentype/spec/gvar)
    fn exec(&self, context: &Context) -> Result<(), Error> {
        // We built the gvar fragments alongside glyphs, now we need to glue them together into a gvar table
        let glyph_order = context.ir.glyph_order.get();

        let variations: Vec<_> = make_variations(&glyph_order, |glyph_name| {
            context
                .gvar_fragments
                .get(&WorkId::GvarFragment(glyph_name.clone()).into())
                .to_deltas()
        });
        let gvar = Gvar::new(variations).map_err(Error::GvarError)?;

        let raw_gvar = dump_table(&gvar)
            .map_err(|e| Error::DumpTableError {
                e,
                context: "gvar".into(),
            })?
            .into();
        context.gvar.set_unconditionally(raw_gvar);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use font_types::F2Dot14;
    use fontir::ir::GlyphOrder;
    use write_fonts::tables::{gvar::GlyphDeltas, variations::Tuple};

    use super::make_variations;

    #[test]
    fn skips_empty_variations() {
        let glyph_with_var = "has_var";
        let glyph_without_var = "no_var";
        let mut glyph_order = GlyphOrder::new();
        glyph_order.insert(glyph_with_var.into());
        glyph_order.insert(glyph_without_var.into());

        let variations = make_variations(&glyph_order, |name| {
            match name.as_str() {
                v if v == glyph_without_var => Vec::new(),
                // At the maximum extent (normalized pos 1.0) of our axis, add +1, +1
                v if v == glyph_with_var => vec![GlyphDeltas::new(
                    Tuple::new(vec![F2Dot14::from_f32(1.0)]),
                    vec![Some((1, 1))],
                    None,
                )],
                v => panic!("unexpected {v}"),
            }
        });

        assert_eq!(1, variations.len(), "{variations:?}");
    }
}
