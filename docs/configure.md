### Конфигурация MBUS-serve
**Конфигурация программы осуществляется редактированием файла config.yaml**

```
sources_data:
  metric_socket: 0.0.0.0:53214
  event_socket: 0.0.0.0:53212
  status_socket: 0.0.0.0:53210

modbus_tcp:
  running: true
  tcp_socket: 127.0.0.1:15223
  unit_id: 1

modbus_rtu:
  running: true
  device: /dev/ttyUSB0
  unit_id: 1
```