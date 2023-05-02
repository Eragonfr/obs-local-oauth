# OBS local oauth

**WARNING ! This project does not work, for now.**

This is an adaptation from [OBS oauth Cloudflare Worker](https://github.com/obsproject/obs-oauth-cf)
to be run on any linux machine.

This will hopefully allow more linux users to run their own oauth proxy, and allow them to use the service integrations in OBS.  

## Limitations

- It is not working yet, please check back later.
- No restream implementation. As I do not use Restream I have no use to implement it's support, any PR to fix that is welcome.
- This is not tested on Windows, MacOS or any *BSD. I don't have any of them, if you have a bug on one of these platform you may report it, but I do not guarantee that I will fix it.

## License

This project is under the GPL-2.0  
The code in `src/platforms/oauth.rs` is under the license of OBS oauth cf.
It's only there as a reference and will be deleted in the next commit
