ALTER TABLE order_items
    DROP CONSTRAINT order_items_code_id_max_tickets_per_user_valid;

DROP FUNCTION order_items_code_id_max_tickets_per_user_valid(UUID, UUID, UUID, BIGINT);

