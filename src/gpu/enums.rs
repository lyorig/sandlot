//! There are a LOT of boolenums in the GPU module.
//! This is a place to aggregate them all, since some are reusable.

use crate::boolenum;

boolenum!(Cycle, "Whether to cycle resources.");
boolenum!(CycleResolveTexture, "Whether to cycle resolve textures.");
boolenum!(
    EnableAlphaToCoverage,
    "Whether to enable the alpha-to-coverage feature."
);
boolenum!(EnableAnisotropy, "Whether to enable anisotropic filtering.");
boolenum!(EnableBlend, "Whether to enable blending.");
boolenum!(
    EnableColorWriteMask,
    "Whether to enable the color write mask."
);
boolenum!(
    EnableCompare,
    "Whether to enable comparison against a reference value."
);
boolenum!(
    EnableDebug,
    "Whether to enable debug mode properties and validations."
);
boolenum!(EnableDepthBias, "Whether to bias fragment depth values.");
boolenum!(
    EnableDepthClip,
    "Whether to enable depth clipping instead of depth clamping."
);
boolenum!(EnableDepthTest, "Whether to enable the depth test.");
boolenum!(EnableDepthWrite, "Whether to enable depth writes.");
boolenum!(EnableStencilTest, "Whether to enable the stencil test.");
boolenum!(
    WaitAll,
    "Whether to wait for all fences, instead of any fence."
);
