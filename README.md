# What is contained in this repository?

This is a silly utility meant to search and re-enable lost drm framebuffers.
This sidesteps an issue on RK3328 implementation where you can get a missing
framebuffer after an application ends ungracefully or fails to properly uwnind
it's modesetting.

Fixes blackscreens on Evercade VS.

# Usage:

- Add `fixvid` to the tail end of your launch scripts.
- Run `fixvid` when screen is gone.

# License

This is free software. The source files in this repository are released under the [Modified BSD License](LICENSE.md), see the license file for more information.