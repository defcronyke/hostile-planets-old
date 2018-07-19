import hpserver

# Instantiate a new Server.
s = hpserver.Server("serverconf.toml")

# Listen on the address that is in the serverconf.toml file.
s.listen()

# Print the contents of the Server instance.
print(dir(s))
