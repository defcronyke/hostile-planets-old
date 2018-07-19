import hpclient

# Instantiate a new Client.
c = hpclient.Client("clientconf.toml")

# Connect to the server listed in the clientconf.toml file.
c.connect()

# Connect to a server with the address given by the first argument.
# c.connect_to("127.0.0.1:8080")

# Print the contents of the Client instance.
print(dir(c))
