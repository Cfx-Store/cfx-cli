
fx_version "cerulean"
game "gta5"
lua54 "yes"

author ""
version "0.0.0"

server_scripts {
     "@oxmysql/lib/MySQL.lua",
     "src/server/main.lua"
}

client_scripts {
     "src/client/main.lua"
}

shared_scripts {
     "@es_extended/imports.lua",
     "@ox_lib/init.lua"
}
        