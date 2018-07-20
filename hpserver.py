import hpserver

# Instantiate a new Server.
s = hpserver.Server("serverconf.toml")

# Print the contents of the Server instance.
print("server contents:")
print(dir(s))

# Listen on the address that is in the serverconf.toml file.
s.listen()
