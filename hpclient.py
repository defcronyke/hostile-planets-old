# Run this script to launch a client and try to connect to a remote 
# server defined in the clientconf.toml config file.
from threading import Thread
import hpclient_vulkan as hpclient
# import hpclient_dx12 as hpclient
# import hpclient_gl as hpclient

# Instantiate a new Client.
c = hpclient.Client("clientconf.toml")

# Connect in the background to the server listed in the clientconf.toml file.
Thread(target=c.connect, daemon=True).start()

# Connect in the background to a server with the address given by the first argument.
# Thread(target=c.connect_to, args=("127.0.0.1:8080",), daemon=True).start()

# Print the contents of the Server instance.
print("client contents:")
print(dir(c))

# Start the client's main loop.
c.run()
