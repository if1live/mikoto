FROM rabbitmq:3.10-management-alpine

COPY rabbitmq.conf /etc/rabbitmq/
COPY enabled_plugins /etc/rabbitmq/

EXPOSE 1883
EXPOSE 5672
EXPOSE 15672
EXPOSE 15674
EXPOSE 15675
EXPOSE 25672
EXPOSE 61613

CMD ["rabbitmq-server"]
