# portproxyd
		Port Proxy Daemon 0.1.0
		SirAlador
		Listens on a TCP port and proxies all traffic to a destination
		
		USAGE:
		    portproxyd [OPTIONS] --listen-on <listen_on> --forward-to <forward_to>
		
		OPTIONS:
		    -b, --buffer-size <buf_size>     The size for each communication buffer. Each active connection
		                                     has two communication buffers
		    -f, --forward-to <forward_to>    The ADDRESS:PORT to forward to
		    -h, --help                       Print help information
		    -l, --listen-on <listen_on>      The [ADDRESS:]PORT to listen on
		    -V, --version                    Print version information
