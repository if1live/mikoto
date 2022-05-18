FROM rabbitmq:3.10-management-alpine

RUN rabbitmq-plugins disable --offline rabbitmq_prometheus
RUN rabbitmq-plugins enable --offline rabbitmq_management
RUN rabbitmq-plugins enable --offline rabbitmq_mqtt
RUN rabbitmq-plugins enable --offline rabbitmq_web_mqtt
RUN rabbitmq-plugins enable --offline rabbitmq_stomp
RUN rabbitmq-plugins enable --offline rabbitmq_web_stomp

# 256MB에서 쓰려면 메모리 기준선을 늘려야한다
# 256MB에서 vm_memory_high_watermark.relative 기본값 0.4가 적용되면
# memory high watermark 경고 뜨고 rabbitmq가 외부의 요청을 무시한다
RUN echo 'vm_memory_high_watermark.relative=0.8' >> /etc/rabbitmq/rabbitmq.conf

RUN cat /etc/rabbitmq/rabbitmq.conf

EXPOSE 1883
EXPOSE 5672
EXPOSE 15672
EXPOSE 15674
EXPOSE 15675
EXPOSE 25672
EXPOSE 61613

CMD ["rabbitmq-server"]
