# Run this script to launch a server. You can specify if it should
# accept remote connections or not in the serverconf.toml config file.
import hpserver

# Instantiate a new Server.
s = hpserver.Server("serverconf.toml")

# Print the contents of the Server instance.
print("server contents:")
print(dir(s))

# Listen on the address that is in the serverconf.toml file.
s.listen()

# Listen on the address specified by the first argument.
# s.listen_to("127.0.0.1:8080")
