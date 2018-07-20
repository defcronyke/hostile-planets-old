import hpclient
from threading import Thread

# Instantiate a new Client.
c = hpclient.Client("clientconf.toml")

# Connect in the background to the server listed in the clientconf.toml file.
Thread(target=c.connect, daemon=True).start()

# Connect in the background to a server with the address given by the first argument.
# Thread(target=c.connect_to, args=("127.0.0.1:8080",), daemon=True).start()

# Start the client's main loop.
c.run()
