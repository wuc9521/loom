SyGuS translations of ../alpharegex.

The original alpharegex files remain in ../alpharegex. In these copies, source `_` positions are concrete `_` characters matched only by `re.allchar`, because the grammar does not include `(str.to_re "_")`. Source `?` positions are expanded over both `0` and `1`.
