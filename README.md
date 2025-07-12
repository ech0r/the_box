# the_box

The Box is a webservice that hosts a nix config for known devices like raspberry pi.

There is a backend written in Actix (rust) and a front-end written in Yew (rust). The database component is sled (rust). 

There should be login and session management, the login provider is GitHub. 

There should be a login page/component, and a dashboard where users can view and edit their nix config. Which is then updated and stored in the backend.

There should be a public endpoint that serves the nix configuration it should be something like /config/{hash} -> where hash is the unique id for their configuration.

Please make the front-end totally responsive with a clean design. Cyberpunk design style if possible.
