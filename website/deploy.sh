# This script is dedicated to the official documentation site at https://dystroy.org/bacon

# build the documentation site
mkdocs build

# copy the site to the deployement stage
cp -r site/* ~/dev/www/dystroy/bacon/

# build the config schema
bacon --generate-config-schema > ~/dev/www/dystroy/bacon/.bacon.schema.json

# deploy on dystroy.org
~/dev/www/dystroy/deploy.sh
