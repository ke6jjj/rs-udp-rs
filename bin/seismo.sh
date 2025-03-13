#!/bin/sh
#
# PROVIDE: seismo
# REQUIRE: LOGIN
# KEYWORD: shutdown

#
# Add the following line to /etc/rc.conf to enable seismo alarm.
# seismo_enable (bool):    Set to "NO" by default.
#                          Set it to "YES" to enable seismo alarm.
# seismo__user (str):      User to run as
# seismo_pidfile (str):    Custom PID file path and name.
#                          Default to "/var/run/seismo.pid".
# seismo_conf (str):       Configuration file.
#                          Default to "/usr/local/etc/seismo.conf"
# seismo_debug (bool):     Keep stdout/stderr attached to running
#                          terminal, for debugging, when starting.
#                          Defaults to "NO"
#

. /etc/rc.subr

name="seismo"
rcvar=seismo_enable

load_rc_config $name

: ${seismo__user:="nobody"}
: ${seismo_debug:="NO"}
: ${seismo_executable:="/usr/local/bin/seismo"}
: ${seismo_pidfile:="/var/run/seismo.pid"}
: ${seismo_conf:="/usr/local/etc/seismo/seismo.config"}
: ${seismo_env_file:=""}

pidfile="${seismo_pidfile}"
command="/usr/sbin/daemon"
command_args="-c -P ${pidfile} -u ${seismo__user}"
process_args="-c ${seismo_conf}"
if [ "${seismo_debug}" == "NO" ]; then
        command_args="${command_args} -f"
fi

if [ ! -z "${seismo_env_file}" -a -f "${seismo_env_file}" ]; then
	. ${seismo_env_file} || echo 2>&1 "can't read" "${seismo_env_file}"
fi

command_args="${command_args} ${seismo_executable} ${process_args}"

# Customized kill command to send signal to entire process group.
_run_rc_killcmd()
{
        local _cmd

        _cmd="kill -$1 -- -$rc_pid"
        if [ -n "$_user" ]; then
                _cmd="su -m ${_user} -c 'sh -c \"${_cmd}\"'"
        fi
        echo "$_cmd"
}

run_rc_command "$1"
