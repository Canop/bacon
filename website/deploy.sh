# This script is dedicated to the official documentation site at https://dystroy.org/bacon

# build the documentation site
ddoc

# build the config schema, so TOML editors pick it up from the site
bacon --generate-config-schema > site/.bacon.schema.json

# deploy directly on the server: going through ~/dev/www/dystroy would republish
# that machine's stale copy of every other project
chmod -R a+rX site
rsync -av site/ dys@dystroy.org:prod/www.dystroy.org/bacon/
