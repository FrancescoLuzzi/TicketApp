project_name="ticket_app_dev"

## env environment modifiers
# POSTGRES_USER="user"
# POSTGRES_PASSWORD="password"
# POSTGRES_DB="db"
# POSTGRES_PORT=5432
# REDIS_PORT=6379

ask_to_close() {
    echo "dev environment already running"
    while true; do
        read -p "Do you want to take it down? (y/n) " yn
        case $yn in 
            [yY] )
                docker compose -p $project_name down
                exit 0
                ;;

            [nN] )
                echo exiting...
                exit 0
                ;;

            * )
                echo invalid response
                ;;
        esac
    done
}

already_running=$(docker compose ls --filter name="$project_name" | wc -l)
((already_running--))

if [ $already_running -eq 1 ]; then
    if [[ $1 = "exec" && ( $2 = "redis" || $2 = "db" ) ]];then
    docker exec -it "${project_name}-${2}-1" bash
    else
        ask_to_close
    fi

    exit 1
fi

docker compose -p $project_name up -d
