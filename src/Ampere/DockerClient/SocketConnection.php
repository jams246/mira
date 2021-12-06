<?php

namespace App\Ampere\DockerClient;

use App\Ampere\DockerClient\Request\Context;
use App\Ampere\DockerClient\Response\Response;

class SocketConnection
{
    private string $socketAddr = 'unix:///var/run/docker.sock';
    private int $connectTimeout = 30;

    private $conn;
    private int $errorCode = 0;
    private string $errorMessage = '';

    private function __construct(private string $context, bool $isBlocking)
    {
        try {
            $this->conn = \stream_socket_client(
                $this->socketAddr,
                $this->errorCode,
                $this->errorMessage,
                $this->connectTimeout,
                STREAM_CLIENT_CONNECT
            );
            \stream_set_blocking($this->conn, $isBlocking);
        } catch (\ErrorException $exception) {
            //Assume that the docker sock file was not found
        }
    }

    public static function create(Context $context)
    {
        return new SocketConnection(
            $context->getContext(),
            $context->isStreamBlocked()
        );
    }

    public function request(): Response
    {
        \stream_socket_sendto($this->conn, $this->context);

        return new Response(\stream_get_contents($this->conn, -1));
    }

    public function startStream(): void
    {
        \stream_socket_sendto($this->conn, $this->context);
    }

    public function getConn()
    {
        return $this->conn;
    }

    public function closeConn()
    {
        \fclose($this->conn);
    }
}
