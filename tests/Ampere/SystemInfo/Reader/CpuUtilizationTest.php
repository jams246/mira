<?php

namespace App\Tests\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\Reader\CpuUtilization;
use Symfony\Component\Cache\Adapter\AdapterInterface;
use Symfony\Component\Serializer\SerializerInterface;

class CpuUtilizationTest extends ReaderHelper
{
    private AdapterInterface $cache;
    private SerializerInterface $serializer;

    public function setUp(): void
    {
        $this->cache = $this->createMock(AdapterInterface::class);
        $this->serializer = $this->createMock(SerializerInterface::class);

        parent::setUp();
    }

    public function testParseResponseIsArray(): void
    {
        $fixtureContent = $this->loadFixture('cpu_stat');

        $object = new CpuUtilization($this->cache, $this->serializer);
        $response = $object->parse($fixtureContent);

        $this->assertIsArray($response);
    }

    public function testCalculateCpuUtilization(): void
    {
        $object = new CpuUtilization($this->cache, $this->serializer);

        $readings = \array_map('floatval', [177418, 119, 264385, 16322496, 903, 0, 15001, 0, 0, 0]);

        $cpuUtilization = $object->calculateCpuUtilization($readings);

        $this->assertIsFloat($cpuUtilization);
        $this->assertEquals(3.0, $cpuUtilization);
    }
}
