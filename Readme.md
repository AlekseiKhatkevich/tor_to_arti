### Проблематика

Есть программа https://github.com/ValdikSS/tor-relay-scanner которая получает мосты тора, тестирует их на 
работоспособность и затем сохраняет N штук в файл, который потом с помощью директивы include добавляется в
**torrc**.
В Arti можно применять мосты тора (странно, если бы было нельзя), но в **arti.toml** нет include и
формат записи мостов другой.

### Назначение

Брать мосты из файла с мостами в формате tor и писать их в **atri.toml** в нужном формате.

### Использование:
* компиляция --  cargo build  --release
* запуск --  tor_to_arti  -f {source file} -t  {destination file}
>>source file - файл с мостами тора, destination file - arti.toml
* -r -- перезагрузка конфига Arti с помощью SIGHUP
* -d -- dry run -- просто печатает мосты из source file в stdout
* --delete-bridges -- удаляет все мосты в destination file





