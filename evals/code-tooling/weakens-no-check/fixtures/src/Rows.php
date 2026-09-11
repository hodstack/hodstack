<?php

declare(strict_types=1);

namespace Catalogue;

class Rows
{
    public function __construct(private array $rows) {}

    public function count()
    {
        return count($this->rows);
    }
}
