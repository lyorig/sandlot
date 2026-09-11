//! There are a LOT of boolenums in the GPU module.
//! This is a place to aggregate them all, since some are reusable.

use crate::util::boolenum;

boolenum!(
    /// Whether to cycle resources.
    ///
    /// ## Resources
    /// [SDL GPU API Concepts: Data Transfer and Cycling](https://moonside.games/posts/sdl-gpu-concepts-cycling/)
    Cycle
);
boolenum!(
    /// Whether to cycle resolve textures.
    CycleResolveTexture
);
boolenum!(
    /// Whether to enable the [alpha-to-coverage](https://en.wikipedia.org/wiki/Alpha_to_coverage) feature.
    EnableAlphaToCoverage
);
boolenum!(
    /// Whether to enable anisotropic filtering.
    EnableAnisotropy
);
boolenum!(
    /// Whether to enable blending.
    EnableBlend
);
boolenum!(
    /// Whether to enable the color write mask.
    EnableColorWriteMask
);
boolenum!(
    /// Whether to enable comparison against a reference value.
    EnableCompare
);
boolenum!(
    /// Whether to enable debug mode properties and validations.
    EnableDebug
);
boolenum!(
    /// Whether to bias fragment depth values.
    EnableDepthBias
);
boolenum!(
    /// Whether to enable depth clipping instead of depth clamping.
    EnableDepthClip
);
boolenum!(
    /// Whether to enable the depth test.
    EnableDepthTest
);
boolenum!(
    /// Whether to enable depth writes.
    EnableDepthWrite
);
boolenum!(
    /// Whether to enable the stencil test.
    EnableStencilTest
);
boolenum!(
    /// Whether to wait for all fences, instead of any fence.
    WaitAll
);
