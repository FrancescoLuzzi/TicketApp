project_name="ticket_app_dev"

## env environment modifiers
POSTGRES_USER="user"
POSTGRES_PASSWORD="password"
POSTGRES_DB="ticket_app"
POSTGRES_PORT=5432
REDIS_PORT=6379

DB_HOST="localhost"

export DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${DB_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}"
echo "export DATABASE_URL=\"$DATABASE_URL\"" > db_env.sh
echo "created file db_env.sh, source it to configure \$DATABASE_URL env variable"
echo "example \". db_env.sh\""

ask_to_close() {
    echo "dev environment already running"
    while true; do
        read -p "Do you want to take it down? (Y/n) " yn
        case $yn in
            y|Y|"" )
                docker compose -p $project_name down
                return 0
                ;;

            n|N )
                echo exiting...
                sqlx database create
                sqlx migrate run
                return 0
                ;;

            * )
                echo invalid response
                ;;
        esac
    done
}

ask_force_recreate() {
    read -p "Force recreation of the containers? (y/N) " yn
    while true; do
        case $yn in
            y|Y )
                docker compose -p $project_name up -d --force-recreate --wait
                break 2
                ;;

            n|N|"" )
                docker compose -p $project_name up -d --wait
                break 2
                ;;

            * )
                echo invalid response
                ;;
        esac
    done
    sqlx database create
    sqlx migrate run
}

already_running=$(docker compose ls --filter name="$project_name" | wc -l)
((already_running--))

if [ $already_running -eq 1 ]; then
    if [[ $1 = "exec" && ( $2 = "redis" || $2 = "db" ) ]];then
        docker exec -it "${project_name}-${2}-1" bash
        exit 0
    else
        ask_to_close
        exit 0
    fi

    exit 1
fi

ask_force_recreate
