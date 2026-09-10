# Sandlot wishlist

- Subsystem handles and references
  - similar to how resources have handles and refs
  - supports derefs, i.e. `Video` can deref to `Events` (since the former inits the latter, according to SDL)
- pass for `Copy` types with methods which unnnecessarily take `&(mut) self` instead of just `(mut) self`
