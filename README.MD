# Skyway

Work-in-progress native bluesky client, designed to adhere to Microsoft's Metro design language as close as is possible or reasonable.

The name is a mix of Blue**sky** and metro -> train -> sub**way**. It's also the (code)name of a map in the Titanfall 2 campaign.

![timeline](https://github.com/headassbtw/skyway/blob/master/screenshots/timeline.png?raw=true)
![profile](https://github.com/headassbtw/skyway/blob/master/screenshots/profile.png?raw=true)
![composer](https://github.com/headassbtw/skyway/blob/master/screenshots/composer.png?raw=true)

## About the codebase

I write things, especially with immediate mode frameworks, as a "get it working and fix it up later" kind of deal.
A lot of this codebase has yet to receive the latter (and might not ever, due to the growing size of it.)

Most notably the parts that need work are:
- Backend to frontend communitcation. A lot of view-specific tasks are sent to the frontend as a whole, this sucks but works for the time being
  - This paradigm is being phased out in favor of sending an `Arc<Mutex<>>` to the backend, which then works on the `Arc`'s contents,
  	but as for view-dependent things I'm still looking for a good way to handle that.
- Repetition. I've got some good things working like UI viewers for posts and the like, but things like embeds, display names, images, etc are very repeated
  - I'm thinking of implementing a global cache for objects like posts, that way likes/reposts update across the entire app, instead of one part (like a thread) having a more up-to-date copy of a post than others (like the timeline)

## Building

This project relies on six Microsoft fonts and other various free fonts for Japanese/etc.
Microsoft's licensing doesn't allow distribution of them, so I won't put them in the source tree.
You will need to provide the Microsoft fonts from your own Windows 8 or newer install, and copy them into the root directory (next to `src`):
- **Segoe UI** `C:/Windows/Fonts/segoeui.ttf`
- **Segoe UI Light**: `C:/Windows/Fonts/segoeuil.ttf`
- **Segoe Symbols**: `C:/Windows/Fonts/seguisym.ttf`
- **Segoe Emojis**: `C:/Windows/Fonts/seguiemj.ttf`
- **Segoe Boot**: `C:/Windows/Boot/Fonts/segoe_slboot.ttf`
- **Malgun Gothic**: `C:/Windows/Fonts/malgun.ttf` (You should be able to substitute this for a Korean font of your choice, but it looks in place (enough) with segoe)
Along with a Japanese font of your choice, I'm using Droid Sans because I know about it
- **Droid Sans Japanese**: `DroidSansJapanese.ttf`