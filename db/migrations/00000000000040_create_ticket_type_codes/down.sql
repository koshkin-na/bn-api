ALTER TABLE ticket_type_codes DROP CONSTRAINT ticket_type_code_ticket_type_id_valid;
DROP FUNCTION ticket_type_code_ticket_type_id_valid;

DROP INDEX index_ticket_type_codes_ticket_type_id_code_id;
DROP TABLE ticket_type_codes;
