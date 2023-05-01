# Distributed Display Protocol (DDP) in Rust

This package allows you to write pixel data to a LED strip over [DDP](http://www.3waylabs.com/ddp/)
Currently implements sending but haven't gotten to implementing response parsing, works for most use cases though.

You can use this to stream pixel data to [WLED](https://github.com/Aircoookie/WLED) or any other DDP capable reciever.

## Why?

I wish I could tell you. I've gone back and forth on these bespoke LED protocols and DDP seems like the most "sane" one although the "specification" is not great. [TPM2.net](https://gist.github.com/jblang/89e24e2655be6c463c56) was another possible protocol which [i started to implement](https://github.com/coral/tpm2net) but stopped after I realized how bad it is. Artnet and E1.31 is great but then you have framerate problem (approx 40-44 FPS) to maintain backwards compatbility with DMX. DDP sits in the middle here as "sane" but not perfect, hence why I implemented it for whatever it is I'm doing. For any future "i'm going to invent my own LED protocol" people out there, take note from the people in broadcast video instead of your jank ham radio serial protocol.

## Contributing

m8 just open a PR with some gucchimucchi code and I'll review it.

![KADSBUGGEL](https://raw.githubusercontent.com/coral/fluidsynth2/master/kadsbuggel.png)