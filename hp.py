# Run this script to launch both a local server and a client which will connect to it.
from threading import Thread
import hpserver
import hpclient_vulkan as hpclient
# import hpclient_dx12 as hpclient
# import hpclient_gl as hpclient

# Instantiate a new Server.
s = hpserver.Server("serverconf.toml")

# Listen in the background on the address that is in the conf.toml file.
Thread(target=s.listen, daemon=True).start()

# Listen in the background on the address specified by the first argument.
# Thread(target=s.listen_to, args=("127.0.0.1:8080",), daemon=True).start()

# Print the contents of the Server instance.
print("server contents:")
print(dir(s))

# Print the server configuration.
print("server conf:")
print(s.get_conf())

# Print the server name.
print("server name: " + s.get_name())

# Print the list of connected players.
print("server players:")
print(s.get_players())

# Check if a player is connected.
player = "default player"
print("is " + player + " connected? " + str(s.is_connected(player)))

# Check if another player is connected.
player = "Henry"
print("is " + player + " connected? " + str(s.is_connected(player)))

# Instantiate a new Client.
c = hpclient.Client("clientconf.toml")

# Connect in the background to the server listed in the clientconf.toml file.
Thread(target=c.connect, daemon=True).start()

# Connect in the background to a server with the address given by the first argument.
# Thread(target=c.connect_to, args=("127.0.0.1:8080",), daemon=True).start()

# Print the contents of the Client instance.
print("client contents:")
print(dir(c))

# Print the client configuration.
print("client conf:")
print(c.get_conf())

# Load a 3D model that's in glTF format.
# c.load_gltf("hpclient/assets/models/cube/cube.glb")

# Start the client's main loop.
c.run()
